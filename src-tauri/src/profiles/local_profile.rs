use std::{fs, io};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::installer::{InstallerConfig, InstallerError};
use crate::launcher::{LauncherProfile, LauncherProfiles};
use crate::mods::Mod;
use crate::profiles::{Profile, ProfileAddon, SFTP_PROFILES_DIR};
use crate::profiles::remote_profile::RemoteProfile;
use crate::resource_packs::ResourcePack;
use crate::sftp::{copy_dir_all, sftp_list_dir};

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalProfile{
    pub name:String,
    pub version:Option<String>,
    pub mods:Option<Vec<Mod>>,
    pub launcher_profile:Option<LauncherProfile>,
    pub resource_packs:Option<Vec<ResourcePack>>,
    pub config:Option<Vec<String>>
}


impl LocalProfile{
    pub fn upload_profile(self)-> Result<RemoteProfile,InstallerError>{
        let mut remote_profile = RemoteProfile::from(self.clone());
        remote_profile.scaffold()?;
        remote_profile.write_launcher_profile()?;
        remote_profile.save_profile()?;
        let mods = self.mods.clone().unwrap();
        self.upload_mods(mods)?;
        Ok(remote_profile)
    }
    pub fn upload_mods(self,mods_list:Vec<Mod>)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let local_mods_path = installer_config.default_game_dir.unwrap().join("profiles").join(self.name);
        for a in mods_list.iter(){
            a.upload(&local_mods_path)?;
        };
        Ok(())
    }
    pub fn install_mods(&mut self,mods_list:Vec<&str>)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open().unwrap();
        let profile_path = installer_config.default_game_dir.unwrap();
        let local_path = profile_path.join("profiles").join(&self.name);
        let mut mods = self.mods.clone().unwrap();
        for a in mods_list.iter(){
            let current_mod = Mod::open_remote(a)?;
            current_mod.download(&local_path)?;
            mods.push(current_mod);
        }
        self.mods = Some(mods);
        self.save_profile()?;
        Ok(())
    }
    pub fn install_new_mods(&mut self,mods_list:Vec<Mod>)->Result<(),InstallerError>{
                let game_dir = InstallerConfig::open()?.default_game_dir.unwrap();
                let mut installed_mods_list = self.mods.clone().unwrap();
                let mut dependencies:HashSet<String>= HashSet::new();
                for x in mods_list {
                    let mut file = File::open(&x.location)?;
                    let new_mod = x.clone();
                    dependencies.extend(x.dependencies);
                    let mut new_file = File::create(game_dir.join("profiles").join(&self.name).join("mods").join(x.file_name))?;
                    io::copy(&mut file, &mut new_file)?;
                    installed_mods_list.push(new_mod);
                }
                self.mods = Some(installed_mods_list);
                match self.find_missing_dependencies(dependencies) {
                    None => {
                        println!("No missing dependencies")
                    }
                    Some(set) => {
                        println!("There are {} dependencies missing!",set.len());
                        let val = set.iter().map(|item| item.as_str()).collect();
                        self.install_mods(val)?;
                    }
                };

                self.save_profile()?;
                Ok(())
            }
            pub fn find_missing_dependencies(&self,dependencies:HashSet<String>)->Option<HashSet<String>>{
                let mut missing_dependencies = dependencies;
                let mod_list = self.mods.clone().unwrap();
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
    pub fn add_resource_pack(&mut self,pack_name:&str)->Result<(),InstallerError>{
        let rp = ResourcePack::open_remote(pack_name)?;
        let mut packs = self.resource_packs.clone().unwrap();
        packs.push(rp.clone());
        let profile_path = InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(&self.name);
        rp.download(&profile_path)?;
        self.resource_packs = Some(packs);
        self.save_profile()?;
        Ok(())
    }

    fn read_mods(&mut self)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let profile_path = PathBuf::from(installer_config.default_game_dir.unwrap()).join("profiles").join(&self.name);
        let mods = fs::read_dir(profile_path.join("mods").as_path())?;
        let mut mod_names = Vec::new();
        for x in mods {
            mod_names.push(Mod::open_local(x.unwrap().file_name().to_str().unwrap()).unwrap());
        };
        self.mods = Some(mod_names);
        Ok(())
    }
    pub fn delete_resource_pack(&mut self,pack_name:&str)->Result<(),InstallerError>{
        let resource_packs_dir = InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(&self.name).join("resourcepacks");
        let readout =fs::read_dir(&resource_packs_dir)?;
        let mut packs = self.resource_packs.clone().unwrap();
        for op in readout{
            match op{
                Ok(entry) => {
                    if entry.file_name().to_str().unwrap().contains(pack_name){
                        dbg!(&entry.file_type());
                        match entry.file_type().unwrap().is_dir() {
                            true => {
                                fs::remove_dir_all(&resource_packs_dir.join(entry.file_name())).unwrap()
                            }
                            false => {
                                fs::remove_file(&resource_packs_dir.join(entry.file_name())).unwrap()
                            }
                        }
                        let index = packs.iter().position(|m| m.name == pack_name).unwrap();
                        packs.remove(index);
                    }
                }
                Err(_) => {}
            }

        }
        self.resource_packs = Some(packs);
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
        &self.read_mods()?;
        &self.read_resource_packs()?;
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
        // let installer_config = InstallerConfig::open()?;
        match LocalProfile::read_profile_manifest(profile_name) {
            Ok(profile) => {
                Ok(profile)
            }
            Err(_) => {
                let mut profile = Self::new(profile_name);
                profile.read_mods()?;
                profile.read_resource_packs()?;
                profile.read_launcher_profile()?;
                profile.save_profile()?;
                Ok(profile)
            }
        }

    }
    fn copy(self, copy_name: &str) -> Result<Self, InstallerError> {
        let installer_config = InstallerConfig::open().unwrap();
        let base_path = installer_config.default_game_dir.unwrap();
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

    fn read_resource_packs(&mut self) -> Result<(), InstallerError> {
        let installer_config = InstallerConfig::open()?;
        let profile_path = PathBuf::from(installer_config.default_game_dir.unwrap()).join("profiles").join(&self.name);
        let mods = fs::read_dir(profile_path.join("resourcepacks").as_path())?;
        let mut resource_packs = Vec::new();
        for x in mods {
            let entry = x.unwrap().file_name().to_str().unwrap().to_string();
            let rp = ResourcePack::open_local(&entry)?;
            resource_packs.push(rp);
        };
        self.resource_packs = Some(resource_packs);
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
    use crate::installer::InstallerConfig;
    use crate::profiles::local_profile::LocalProfile;
    use crate::profiles::Profile;


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
        dbg!(&result);
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
        let profile_name = "new_profile";
        let profile = LocalProfile::open(profile_name).unwrap();
        let result = profile.upload_profile();
        assert!(&result.is_ok());
        let remote_profile = result.unwrap();
        dbg!(&remote_profile);

    }
    #[test]
    #[serial]
    fn test_upload_mods(){
        let profile_name = "new_profile";
        let new_profile = LocalProfile::open(profile_name).unwrap();
        let mods = new_profile.mods.clone().unwrap();
        let result = new_profile.upload_mods(mods);
        dbg!(&result);
        assert!(result.is_ok());
    }
    #[test]
    fn test_read_resource_packs(){
        let profile_name = "new_profile";
        let mut new_profile = LocalProfile::new(profile_name);
        let result =new_profile.read_resource_packs();
        assert!( result.is_ok());
        assert_eq!(new_profile.resource_packs.unwrap().len(),2)
    }
    #[test]
    fn test_delete_resource_pack(){
        let profile_name = "new_profile";
        let mut local_profile = LocalProfile::open(profile_name).unwrap();
        let result = local_profile.delete_resource_pack("deleteme");
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
        let result = &local_profile.install_mods(first_mod);
        println!("{:?}",local_profile);
        assert!(result.is_ok());
        assert_eq!(local_profile.mods.as_ref().unwrap().len(), 1);

        // let second_mods = Vec::from(["testjar.jar"]);
        // let _ = &local_profile.install_mods(second_mods);
        // assert_eq!(local_profile.mods.as_ref().unwrap().len(),2);

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
}