use std::{fs, io};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::addons::{AddonType, ProfileAddon};
use crate::installer::{InstallerConfig, InstallerError};
use crate::launcher::{LauncherProfile, LauncherProfiles};
use crate::profiles::{Profile};
use crate::profiles::remote_profile::RemoteProfile;
use crate::sftp::{copy_dir_all};

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalProfile{
    pub name:String,
    pub version:Option<String>,
    pub mods:Option<Vec<ProfileAddon>>,
    pub launcher_profile:Option<LauncherProfile>,
    pub resource_packs:Option<Vec<ProfileAddon>>,
    pub config:Option<Vec<String>>
}


impl LocalProfile{
    pub fn upload_profile(self)-> Result<RemoteProfile,InstallerError>{
        let mut remote_profile = RemoteProfile::from(self.clone());
        remote_profile.scaffold()?;
        remote_profile.write_launcher_profile()?;
        remote_profile.save_profile()?;
        &self.upload_addons(AddonType::Mod)?;
        &self.upload_addons(AddonType::ResourcePack)?;
        Ok(remote_profile)
    }
    pub fn upload_addons(&self,addons_type:AddonType)->Result<(),InstallerError>{
        for a in self.get_type_addons(addons_type)?.iter(){
            a.upload(&addons_type.get_local_dir(&self.name)?)?;
        };
        let rp =RemoteProfile::from(self.clone());
        rp.save_profile()?;
        Ok(())
    }

    pub fn upload_specific_addons(self,addons_list:Vec<ProfileAddon>,addon_type:AddonType)->Result<(),InstallerError>{
        for a in addons_list.iter(){
            a.upload(&addon_type.get_local_dir(&self.name).unwrap())?;
        };
        let rp =RemoteProfile::from(self.clone());
        rp.save_profile()?;
        Ok(())
    }
    pub fn install_addons(&mut self,addon_list:Vec<&str>,addon_type: AddonType)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open().unwrap();
        let profile_path = installer_config.default_game_dir.unwrap();
        let local_path = profile_path.join("profiles").join(&self.name);
        let mut dependencies:HashSet<String>= HashSet::new();
        let mut addons:Vec<ProfileAddon> = self.get_type_addons(addon_type).unwrap();
        for a in addon_list.iter(){
            let current_mod = ProfileAddon::open_remote(a,addon_type)?;
            dependencies.extend(current_mod.dependencies.clone());
            current_mod.download(&local_path)?;
            addons.push(current_mod);
        }
        self.set_type_addons(addons,addon_type)?;
        match self.find_missing_dependencies(dependencies){
            None => {}
            Some(set) => {
                let mod_names = set.iter().map(|item| item.as_str()).collect();
                self.install_addons(mod_names,AddonType::Mod)?;
            }
        }
        self.save_profile()?;
        Ok(())
    }
    pub fn install_new_addons(&mut self,mods_list:Vec<ProfileAddon>,addon_type: AddonType)->Result<(),InstallerError>{

        let addons_path = addon_type.get_local_dir(&self.name)?;

        let mut installed_addons = self.get_type_addons(addon_type).unwrap();
        let mut dependencies:HashSet<String>= HashSet::new();

        for x in mods_list {
            let mut file = File::open(&x.location)?;
            let new_mod = x.clone();
            dependencies.extend(x.dependencies);
            let mut new_file = File::create(addons_path.join(x.file_name))?;
            io::copy(&mut file, &mut new_file)?;
            installed_addons.push(new_mod);
        }

        self.set_type_addons(installed_addons,addon_type)?;

        match self.find_missing_dependencies(dependencies) {
            None => {
                println!("No missing dependencies")
            }
            Some(set) => {
                println!("There are {} dependencies missing!",set.len());
                let mod_names = set.iter().map(|item| item.as_str()).collect();
                self.install_addons(mod_names,AddonType::Mod)?;
            }
        };

        self.save_profile()?;
        Ok(())
    }
    pub fn find_missing_dependencies(&self,dependencies:HashSet<String>)->Option<HashSet<String>>{
        let mut missing_dependencies = dependencies;
        let mod_list = self.get_type_addons(AddonType::Mod).unwrap();;
        for x in mod_list {
            match missing_dependencies.contains(&x.name) {
                true => {
                    missing_dependencies.remove(&x.name);
                }
                _ => {}
            }
        }
        return match missing_dependencies.len() {
            0 => None,
            _ => Some(missing_dependencies)
        }
    }
    pub fn install_addon(&mut self,pack_name:&str,addon_type: AddonType)->Result<(),InstallerError>{
        let rp = ProfileAddon::open_remote(pack_name,addon_type)?;
        let mut addons = self.get_type_addons(addon_type)?;
        addons.push(rp.clone());
        rp.download(&addon_type.get_local_dir(&self.name).unwrap())?;
        self.set_type_addons(addons,addon_type)?;
        self.save_profile()?;
        Ok(())
    }


    pub fn delete_addon(&mut self,addon_name:&str,addon_type: AddonType)->Result<(),InstallerError>{
        let addon_dir = addon_type.get_local_dir(&self.name)?;
        let readout =fs::read_dir(&addon_dir)?;
        let mut addons = self.get_type_addons(addon_type)?;
        for addon in readout{
            match addon{
                Ok(entry) => {
                    if entry.file_name().to_str().unwrap().contains(addon_name){
                        dbg!(&entry.file_type());
                        match entry.file_type().unwrap().is_dir() {
                            true => {
                                fs::remove_dir_all(&addon_dir.join(entry.file_name())).unwrap()
                            }
                            false => {
                                fs::remove_file(&addon_dir.join(entry.file_name())).unwrap()
                            }
                        }
                        let index = addons.iter().position(|m| m.name == addon_name).unwrap();
                        addons.remove(index);
                    }
                }
                Err(_) => {}
            }
        }
        self.set_type_addons(addons,addon_type)?;
        self.save_profile()?;
    Ok(())
    }
    pub fn read_profile_manifest<S:Into<String>>(profile_name:S)->Result<Self,InstallerError>{
        let mut file_name = profile_name.into();
        let profile_dir = InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(&file_name);
        file_name.push_str(".config");
        let file = File::open(profile_dir.join(&file_name))?;
        Ok(serde_json::from_reader(file)?)
    }
    pub fn verify_profile_files(&mut self)->Result<(),InstallerError>{
        &self.read_addons(AddonType::Mod)?;
        &self.read_addons(AddonType::ResourcePack)?;
        &self.read_launcher_profile()?;
        &self.save_profile()?;
        Ok(())
    }
    pub fn save_profile(&self)->Result<(),InstallerError>{
        let profile_dir = InstallerConfig::open()?.default_game_dir.unwrap().join("profiles").join(&self.name);
        let mut file_name = self.name.clone();
        file_name.push_str(".config");
        let _ = fs::create_dir(&profile_dir);
        let mut file = File::create(&profile_dir.join(file_name))?;
        file.write(serde_json::to_string_pretty(&self)?.as_ref())?;
        Ok(())
    }


}
impl Profile for LocalProfile{
    fn new(profile_name: &str) -> Self {
        Self {
            name: profile_name.parse().unwrap(),
            mods: None,
            version:None,
            launcher_profile: None,
            resource_packs: None,
            config: None,
        }
    }

    fn create(profile_name: &str)->Result<Self,InstallerError>{
        let profile = Self::new(profile_name);
        let base_path = &InstallerConfig::open().unwrap().default_game_dir.unwrap();
        profile.scaffold()?;
        let launcher_profile =  LauncherProfile::new(&profile_name);
        let mut launcher_profiles = LauncherProfiles::from_file(base_path);
        launcher_profiles.insert_profile(launcher_profile,&profile_name)?;
        Ok(profile)
    }
    fn scaffold(&self)->Result<(),InstallerError>{
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let profile_path = &base_path.join("profiles").join(&self.name);
        fs::create_dir_all(&profile_path.join("mods")).expect("Couldnt create the profile directory");
        fs::create_dir_all(&profile_path.join("resourcepacks")).expect("Couldnt create the profile directory");
        fs::copy(&base_path.join("options.txt"),&profile_path.join("options.txt")).expect("Could not create options copy");
        fs::copy(&base_path.join("servers.dat"),&profile_path.join("servers.dat")).expect("Could not create options copy");
        Ok(())
    }
    fn open(profile_name:&str) -> Result<Self, InstallerError> {
        match LocalProfile::read_profile_manifest(profile_name) {
            Ok(profile) => {
                Ok(profile)
            }
            Err(_) => {
                let mut profile = Self::new(profile_name);
                profile.read_addons(AddonType::Mod)?;
                profile.read_addons(AddonType::ResourcePack)?;
                profile.read_launcher_profile()?;
                profile.save_profile()?;
                Ok(profile)
            }
        }

    }
    fn copy(self, copy_name: &str) -> Result<Self, InstallerError> {
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let mut new_profile = LocalProfile::new(copy_name);
        copy_dir_all(base_path.join("profiles").join(self.name),base_path.join("profiles").join(copy_name))?;
        let mut new_launcher_profile = self.launcher_profile.clone().unwrap();
        new_launcher_profile.name = Some(copy_name.to_string());
        new_profile.launcher_profile= Some(new_launcher_profile);
        new_profile.mods = self.mods.clone();
        new_profile.config = self.config.clone();
        new_profile.resource_packs = self.resource_packs.clone();
        new_profile.write_launcher_profile()?;
        Ok(new_profile)
    }
    fn delete(self) -> Result<(), InstallerError> {
        let profile_path = InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(&self.name);
        fs::remove_dir_all(profile_path)?;
        LauncherProfiles::open().remove_profile(&self.name)?;
        Ok(())
    }

    fn read_addons(&mut self,addon_type: AddonType)->Result<(),InstallerError>{
        let addon_dir = addon_type.get_local_dir(&self.name)?;
        let readout = fs::read_dir(addon_dir.as_path())?;
        let mut addons = Vec::new();
        for x in readout {
            addons.push(ProfileAddon::open_local(x.unwrap().file_name().to_str().unwrap(),addon_type).unwrap());
        };
        self.set_type_addons(addons,addon_type)?;
        Ok(())
    }

    fn get_type_addons(&self, addon_type: AddonType)->Result<Vec<ProfileAddon>,InstallerError>{
        return match addon_type{
            AddonType::ResourcePack => {
                Ok(self.resource_packs.clone().unwrap_or_else(|| Vec::new()))
            }
            AddonType::Mod => {
                Ok(self.mods.clone().unwrap_or_else(|| Vec::new()))
            }
        };
    }
    fn set_type_addons(&mut self, addons:Vec<ProfileAddon>, addon_type: AddonType) ->Result<(),InstallerError>{
        match addon_type{
            AddonType::ResourcePack => {
                self.resource_packs = Some(addons);
            }
            AddonType::Mod => {
                self.mods = Some(addons);
            }
        }
        Ok(())
    }


    fn write_launcher_profile(&mut self) -> Result<(), InstallerError> {
        let mut launcher_profiles = LauncherProfiles::from_file(&InstallerConfig::open()?.default_game_dir.unwrap());
        let _ = launcher_profiles.profiles.remove(&self.name);
        launcher_profiles.profiles.insert(self.name.clone(),self.launcher_profile.clone().unwrap());
        launcher_profiles.save();
        Ok(())
    }
    fn read_launcher_profile(&mut self) -> Result<(), InstallerError> {
        let mut launcher_profiles = LauncherProfiles::from_file(&InstallerConfig::open()?.default_game_dir.unwrap());
        self.launcher_profile = launcher_profiles.profiles.remove(&self.name);
        Ok(())
    }

    fn rename_profile(&mut self, new_name:&str) -> Result<(), InstallerError>{
        let base_path = &InstallerConfig::open()?.default_game_dir.unwrap();
        println!("{:?}",base_path);
        println!("{:?}",base_path.join("profiles").join(&self.name));
        println!("{:?}",base_path.join("profiles").join(new_name));
        // Rename profile directory
        fs::rename(base_path.join("profiles").join(&self.name),base_path.join("profiles").join(new_name)).expect("Failed");
        // Rename launcher_profiles entry
        let mut launcher_profiles = LauncherProfiles::from_file(base_path);
        match launcher_profiles.profiles.remove(&self.name) {
            None => {}
            Some(launcher_profile) => {
                launcher_profiles.insert_profile(launcher_profile,new_name)?;
                launcher_profiles.save();
            }
        };
        self.name= new_name.to_string();
        Ok(())
    }
}
#[cfg(test)]
mod test{
    use std::fs;
    use std::fs::File;
    use serial_test::serial;
    use crate::addons::{AddonManager, AddonType, ProfileAddon};
    use crate::installer::{InstallerConfig, InstallerError};
    use crate::profiles::local_profile::LocalProfile;
    use crate::profiles::{Profile};


    #[test]
    #[serial]
    fn test_new_local_profile(){
        let profile_name = "new_profile";
        let new_profile = LocalProfile::new(profile_name);
        new_profile.scaffold().unwrap();
        assert_eq!(new_profile.name, profile_name)
    }
    #[test]
    fn test_open_profile(){
        let profile_name = "jman_modpack";
        let result = LocalProfile::open(profile_name);
        assert!(result.is_ok());
    }
    #[test]
    fn test_verify_profile_files(){
        let profile_name = "new_profile";
        let game_path = InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(profile_name);
        let mut profile = LocalProfile::open(profile_name).unwrap();
        profile.mods =Some(Vec::new());
        profile.scaffold().unwrap();
        File::create(&game_path.join("mods").join("testjar.jar")).expect("Could not create test jar");
        profile.verify_profile_files().unwrap();
        dbg!(&profile);
        assert_eq!(profile.mods.unwrap().len(), 1);
    }
    #[test]
    fn test_upload_profile(){
        let mut local_profile:LocalProfile = LocalProfile::open("test_profile").or_else(|e| setup_test_mods()).unwrap();
        let result = local_profile.upload_profile();
        assert!(&result.is_ok());
        let remote_profile = result.unwrap();

    }
    #[test]
    #[serial]
    fn test_upload_mods(){
        let mut local_profile:LocalProfile = LocalProfile::open("test_profile").or_else(|e| setup_test_mods()).unwrap();
        let result = local_profile.upload_addons(AddonType::Mod);
        assert!(result.is_ok());
    }
    #[test]
    fn test_read_resource_packs(){
        let profile_name = "new_profile";
        let mut new_profile = LocalProfile::new(profile_name);
        let result = new_profile.read_addons(AddonType::ResourcePack);
        assert!( result.is_ok());
        assert_eq!(new_profile.resource_packs.unwrap().len(),1)
    }
    #[test]
    fn test_delete_resource_pack(){
        let profile_name = "new_profile";
        let mut local_profile = LocalProfile::open(profile_name).unwrap();
        let result = local_profile.delete_addon("deleteme",AddonType::ResourcePack);
        dbg!(&result);
        assert!(result.is_ok());

    }
    #[test]
    #[serial]
    fn test_install_mods(){
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let profile_name = "new_profile";
        let _ = fs::remove_dir_all(base_path.join("profiles").join(profile_name).join("mods"));
        let _ = fs::create_dir(base_path.join("profiles").join(profile_name).join("mods"));
        let mut local_profile = LocalProfile::open(profile_name).unwrap();
        local_profile.mods = Some(Vec::new());
        let first_mod = Vec::from(["optifine"]);
        let result = &local_profile.install_addons(first_mod,AddonType::Mod);
        println!("{:?}",local_profile);
        assert!(result.is_ok());
        assert_eq!(local_profile.mods.as_ref().unwrap().len(), 1);


    }
    #[test]
    #[serial]
    fn test_create_profile(){
        let profile_name = "create_profile";
        let new_profile = LocalProfile::create(profile_name).unwrap();
        println!("{:?}",new_profile);
    }
    #[test]
    #[serial]
    fn test_local_profile_read_launcher(){
        let mut local = LocalProfile::new("new_profile");
        let result = local.read_launcher_profile();
        assert!(result.is_ok())
    }
    #[test]
    #[serial]
    fn test_local_profile_rename(){
        let mut local = LocalProfile::new("new_profile");
        let result = local.rename_profile("new_name_profile");
        let installer_config = InstallerConfig::open().unwrap();
        let val = fs::read_dir(installer_config.default_game_dir.unwrap().join("profiles")).unwrap();
        for x in val {
            println!("{:?}",x);
        }
        assert!(result.is_ok());
        let _ = local.rename_profile("new_profile");
    }
    fn setup_test_mods()->Result<LocalProfile,InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let mut local = LocalProfile::new("test_profile");
        local.scaffold()?;
        let profile_path = installer_config.default_game_dir.unwrap().join("profiles").join(&local.name);
        let mut test_mod = ProfileAddon::new("testmod.jar",AddonType::Mod);
        let mut test_dep =  ProfileAddon::new("testdep1.jar",AddonType::Mod);
        let mut test_dep2 =  ProfileAddon::new("testdep2.jar",AddonType::Mod);
        test_mod.dependencies = Vec::from([String::from("testdep1")]);
        test_dep.dependencies = Vec::from([String::from("testdep2")]);
        test_dep2.dependencies = Vec::from([String::from("testdep1")]);
        File::create(profile_path.join("mods").join(&test_mod.file_name)).expect("Could not create mod");
        File::create(profile_path.join("mods").join(&test_dep.file_name)).expect("Could not create dep");
        File::create(profile_path.join("mods").join(&test_dep2.file_name)).expect("Could not create dep2");
        test_mod.upload(&profile_path)?;
        test_dep.upload(&profile_path)?;
        test_dep2.upload(&profile_path)?;
        fs::remove_file(&profile_path.join("mods").join("testmod.jar"))?;
        fs::remove_file(&profile_path.join("mods").join("testdep1.jar"))?;
        fs::remove_file(&profile_path.join("mods").join("testdep2.jar"))?;
        local.save_profile()?;
        return Ok(local);
    }
    #[test]
    fn test_install_new_mods(){
        let mut local_profile:LocalProfile = LocalProfile::open("test_profile").or_else(|e| setup_test_mods()).unwrap();
        let mod_list:Vec<&str> = Vec::from(["testmod"]);
        let install_result = local_profile.install_addons(mod_list,AddonType::Mod);
        assert!(install_result.is_ok());
    }
    // #[test]
    // fn fix_addons(){
    //     AddonManager::fix_all_addons(AddonType::Mod);
    //     AddonManager::fix_all_addons(AddonType::ResourcePack);
    // }
}