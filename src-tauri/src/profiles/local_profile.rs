use std::{fs, io};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::installer::{InstallerConfig, InstallerError};
use crate::launcher::{LauncherProfile, LauncherProfiles};
use crate::profiles::{Profile, SFTP_PROFILES_DIR};
use crate::profiles::remote_profile::RemoteProfile;
use crate::sftp::{copy_dir_all, sftp_list_dir};

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalProfile{
    pub name:String,
    pub mods:Option<Vec<String>>,
    pub launcher_profile:Option<LauncherProfile>,
    pub resource_packs:Option<Vec<String>>,
    pub config:Option<Vec<String>>
}


impl LocalProfile{
    pub fn upload_profile(self)-> Result<(),InstallerError>{
        let mut remote_profile = RemoteProfile::from(self.clone());
        remote_profile.scaffold()?;
        remote_profile.write_launcher_profile()?;
        let mods = self.mods.clone().unwrap();
        self.upload_mods(mods)?;
        Ok(())
    }
    pub fn upload_mods(self,mods_list:Vec<String>)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let sftp =installer_config.sftp_safe_connect().unwrap();
        let remote_mods_path = PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).join("mods");
        let local_mods_path = installer_config.default_game_dir.unwrap().join("profiles").join(self.name).join("mods");
        for a in mods_list.iter(){
            println!("{:?}",a);
            let mut upload_file = fs::File::open(local_mods_path.join(a)).expect("Could not find File!");
            let mut remote_file = sftp.create(remote_mods_path.join(a).as_path()).expect("Could not create File");
            io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
        };
        Ok(())
    }
    pub fn install_mods(&mut self,mods_list:Vec<&str>)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open().unwrap();
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_path = PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).join("mods");
        let profile_path = installer_config.default_game_dir.unwrap();
        let local_path = profile_path.join("profiles").join(&self.name).join("mods");
        let mut mods = self.mods.clone().unwrap();
        match sftp_list_dir(&remote_path.as_path()){
            Ok(readout) => {
                for x in readout {
                    let file_name = x.0.file_name().unwrap().to_str().unwrap();
                    match mods_list.contains(&file_name){
                        true => {
                            let mut remote_file = sftp.open(&remote_path.join(&file_name)).expect("Could not find remote mod File");
                            let mut local_file = fs::File::create(local_path.join(&file_name).as_path()).expect("Could not create local mod file!");
                            io::copy(&mut remote_file, &mut local_file).expect("Could not write file!");
                            mods.push(file_name.to_string());
                        }
                        false => {}
                    }
                };
                self.mods = Some(mods);
                Ok(())
            }
            Err(err) => {
                Err(InstallerError::from(err))
            }
        }
    }
}
impl Profile for LocalProfile{
    fn new(profile_name: &str) -> Self {
        Self {
            name: profile_name.parse().unwrap(),
            mods: None,
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
        println!("{:?}",base_path);
        fs::create_dir_all(&profile_path.join("mods")).expect("Couldnt create the profile directory");
        fs::copy(&base_path.join("options.txt"),&profile_path.join("options.txt")).expect("Could not create options copy");
        fs::copy(&base_path.join("servers.dat"),&profile_path.join("servers.dat")).expect("Could not create options copy");
        Ok(())
    }
    fn open(profile_name:&str) -> Result<Self, InstallerError> {
        // let installer_config = InstallerConfig::open()?;
        let mut profile = Self::new(profile_name);
        profile.read_mods()?;
        profile.read_launcher_profile()?;
        Ok(profile)
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
    fn read_mods(&mut self)->Result<(),InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let profile_path = PathBuf::from(installer_config.default_game_dir.unwrap()).join("profiles").join(&self.name);
        let mods = fs::read_dir(profile_path.join("mods").as_path())?;
        let mut mod_names = Vec::new();
        for x in mods {
            let entry = x.unwrap().file_name().to_str().unwrap().to_string();
            mod_names.push(entry);
        };
        self.mods = Some(mod_names);
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
    use serial_test::serial;
    use crate::installer::InstallerConfig;
    use crate::profiles::local_profile::LocalProfile;
    use crate::profiles::Profile;

    const BASE_PATH_STRING: &str = "test\\.minecraft";
    const SFTP_PROFILES_DIR: &str = "/upload/profiles/";
    #[test]
    #[serial]
    fn test_new_local_profile(){
        let profile_name = "new_profile";
        let new_profile = LocalProfile::new(profile_name);
        assert_eq!(new_profile.name, profile_name)
    }
    #[test]
    #[serial]
    fn test_install_mods(){
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let profile_name = "new_profile";
        let _ = fs::remove_dir_all(base_path.join("profiles").join(profile_name).join("mods"));
        let _ = fs::create_dir(base_path.join("profiles").join(profile_name).join("mods"));
        let mut local_profile = LocalProfile::open(profile_name).unwrap();
        let first_mod = Vec::from(["testjar.jar"]);
        let result = &local_profile.install_mods(first_mod);
        println!("{:?}",local_profile);
        assert!(result.is_ok());
        assert_eq!(local_profile.mods.as_ref().unwrap().len(), 1);

        let second_mods = Vec::from(["testjar.jar"]);
        let _ = &local_profile.install_mods(second_mods);
        assert_eq!(local_profile.mods.as_ref().unwrap().len(),2);

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