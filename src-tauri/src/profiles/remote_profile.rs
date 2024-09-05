use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::addons::{AddonType, ProfileAddon};
use crate::installer::{InstallerConfig, InstallerError};
use crate::launcher::LauncherProfile;
use crate::profiles::local_profile::LocalProfile;
use crate::profiles::{Profile, SFTP_PROFILES_DIR};
use crate::sftp::sftp_remove_dir;


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RemoteProfile{
    pub name:String,
    pub version:Option<String>,
    pub mods:Option<Vec<ProfileAddon>>,
    pub launcher_profile:Option<LauncherProfile>,
    pub resource_packs:Option<Vec<ProfileAddon>>,
    pub config:Option<Vec<String>>,
}
impl From<LocalProfile> for RemoteProfile{
    fn from(value: LocalProfile) -> Self {
        Self{
            name: value.name,
            mods: value.mods,
            version:value.version,
            launcher_profile: value.launcher_profile,
            resource_packs: value.resource_packs,
            config: value.config,
        }
    }
}
impl RemoteProfile{
    pub fn install_profile(self)->Result<LocalProfile,InstallerError>{
        dbg!(&self);
        let mut local_profile = LocalProfile::from(self.clone());
        let mut new_launcher_profile = self.launcher_profile.clone().unwrap_or_else(||LauncherProfile::new(&self.name));
        new_launcher_profile.game_dir = Some(InstallerConfig::open().unwrap().default_game_dir.unwrap().join("profiles").join(&self.name));
        local_profile.launcher_profile = Some(new_launcher_profile);
        local_profile.scaffold()?;
        local_profile.write_launcher_profile()?;
        local_profile.save_profile()?;
        self.install_addons(AddonType::Mod)?;
        Ok(local_profile)
    }
    pub fn install_addons(&self,addon_type: AddonType)->Result<(),InstallerError>{
        let mods_list:Vec<ProfileAddon> = self.get_type_addons(addon_type).unwrap();
        for m in mods_list.iter(){
            m.download(&addon_type.get_local_dir(&self.name)?).unwrap()
        }
        Ok(())
    }
    pub fn save_profile(&self)->Result<(),InstallerError>{
        let sftp = InstallerConfig::open()?.sftp_safe_connect()?;
        let mut file = sftp.create(PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).join("profile.json").as_path())?;
        let self_json = serde_json::to_string_pretty(&self)?;
        file.write(self_json.as_ref())?;
        Ok(())
    }
    pub fn read_profile_manifest<S:Into<String>>(name:S) ->Result<Self,InstallerError>{
        println!("Looking for profile manifest");
        let name = name.into();
        let sftp = InstallerConfig::open()?.sftp_safe_connect()?;
        let file = sftp.open(PathBuf::from(SFTP_PROFILES_DIR).join(name).join("profile.json").as_path())?;
        let profile:RemoteProfile = serde_json::from_reader(file)?;
        println!("Profile manifest found, returning profile");
        Ok(profile)

    }
}
impl Profile for RemoteProfile{
    fn new(profile_name: &str) -> Self {
        Self{
            name:profile_name.parse().unwrap(),
            mods: None,
            version:None,
            launcher_profile: None,
            resource_packs: None,
            config: None,
        }
    }
    fn create(profile_name:&str) ->Result<Self,InstallerError> {
        let new_profile = RemoteProfile::new(profile_name);
        new_profile.scaffold()?;
        // Sets up Remote profile directories

        Ok(new_profile)
    }

    fn scaffold(&self) -> Result<(), InstallerError> {

        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        let profile_path = PathBuf::from(SFTP_PROFILES_DIR).join(&self.name);
        let _ = sftp.mkdir(profile_path.as_path(),1002);
        let _ = sftp.mkdir(&profile_path.join("mods").as_path(), 1000);
        let _ = sftp.mkdir(&profile_path.join("resource_packs").as_path(), 1000);
        let _ = sftp.mkdir(&profile_path.join("config").as_path(), 1000);
        sftp.lstat(profile_path.join("mods").as_path())?;
        Ok(())
    }

    fn open(profile_name: &str) -> Result<Self, InstallerError>
    where
        Self: Sized
    {

        match RemoteProfile::read_profile_manifest(profile_name) {
            Ok(profile) => {
                Ok(profile)
            }
            Err(_) => {
                let mut profile = Self::new(profile_name);
                profile.read_launcher_profile()?;
                let _ = profile.save_profile();
                Ok(profile)
            }
        }

    }

    fn copy(self, copy_name: &str) -> Result<Self,InstallerError> {
        let mut new_profile = RemoteProfile::create(copy_name)?;
        let mut new_launcher_profile = self.launcher_profile.clone().unwrap();
        new_launcher_profile.name = Some(copy_name.to_string());
        new_profile.launcher_profile= Some(new_launcher_profile);
        new_profile.write_launcher_profile()?;
        new_profile.mods = self.mods;
        new_profile.resource_packs = self.resource_packs;

        Ok(new_profile)

    }

    fn delete(self) -> Result<(), InstallerError> {
        let sftp = InstallerConfig::open()?.sftp_safe_connect()?;
        sftp_remove_dir(&PathBuf::from(SFTP_PROFILES_DIR).join(self.name), &sftp)?;
        Ok(())
    }


    fn read_addons(&mut self, addon_type: AddonType) -> Result<(), InstallerError> {
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().expect("Could not establish SFTP connection");
        match sftp.readdir(addon_type.get_remote_dir().as_path()) {
            Ok(dir_readout) => {
                let mut addons:Vec<ProfileAddon> = Vec::new();
                for i in dir_readout.iter() {
                    let file_name = i.0.file_name().unwrap();
                    addons.push(ProfileAddon::open_remote(file_name.to_str().unwrap(),addon_type)?);
                }
                self.set_type_addons(addons,addon_type)?;
                Ok(())
            }
            Err(err) => {
                Err(InstallerError::from(err))
            }
        }
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
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        match sftp.create(PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).join("launcher_profile.json").as_path()) {
            Ok(mut file) => {
                file.write(serde_json::to_string(&self.launcher_profile)?.as_ref())?;
                Ok(())
            }
            Err(err) => {
                Err(InstallerError::Ssh2(err))
            }
        }
    }

    fn read_launcher_profile(&mut self) -> Result<(), InstallerError> {
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        match sftp.open(PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).join("launcher_profile.json").as_path()) {
            Ok(file) => {
                self.launcher_profile = serde_json::from_reader(file)?;
                Ok(())
            }
            Err(err) => {
                Err(InstallerError::Ssh2(err))
            }
        }
    }
    fn rename_profile(&mut self, new_name: &str) -> Result<(), InstallerError> {
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        sftp.rename(PathBuf::from(SFTP_PROFILES_DIR).join(&self.name).as_path(), PathBuf::from(SFTP_PROFILES_DIR).join(new_name).as_path(), None)?;
        self.name = new_name.to_string();
        let mut new_launcher_profile = self.launcher_profile.clone().unwrap();
        new_launcher_profile.name=Some(new_name.to_string());
        self.launcher_profile=Some(new_launcher_profile);
        self.write_launcher_profile()?;
        Ok(())
    }
}

#[cfg(test)]
mod test{
    use std::fs;
    use std::path::PathBuf;
    use serial_test::serial;
    use crate::addons::AddonType;
    use crate::installer::InstallerConfig;
    use crate::profiles::local_profile::LocalProfile;
    use crate::profiles::{Profile, SFTP_PROFILES_DIR};
    use crate::profiles::remote_profile::RemoteProfile;
    use crate::sftp::{sftp_list_dir, sftp_read_launcher_profile};

    #[test]
    fn test_open_remote_profile(){
        let profile_name = "new_profile";
        let result = RemoteProfile::open(profile_name);
        dbg!(&result);
        assert!(result.is_ok())

    }
    #[test]
    #[serial]
    fn test_write_launcher_profile(){
        let profile_name="new_profile";
        let new_profile_name="new_profile_name";
        let mut result = RemoteProfile::open(profile_name).unwrap();
        assert_eq!(result.launcher_profile.clone().unwrap().name.unwrap(),profile_name.to_string());
        result.rename_profile(new_profile_name).unwrap();
        let written_profile = sftp_read_launcher_profile(new_profile_name).unwrap();
        assert_eq!(written_profile.name.unwrap(),"new_profile_name".to_string());

        // Revert profile to profile_name
        result.rename_profile(profile_name).unwrap();
    }
    #[test]
    #[serial]
    fn test_copy_profile(){
        let profile_name = "new_profile";
        {
            let initial_profile = RemoteProfile::open(profile_name).unwrap();
            let new_profile = initial_profile.copy("copy_profile").unwrap();
            println!("{:?}",new_profile);
            let readout = sftp_list_dir(PathBuf::from(SFTP_PROFILES_DIR).as_path()).unwrap();
            println!("{:?}",readout);
        }
        // let result = RemoteProfile::open(profile_name);
        // assert!(result.is_err());
        // RemoteProfile::new(profile_name);
    }
    #[test]
    fn test_install_profile(){
        let profile_name = "new_profile";
        let remote_profile = RemoteProfile::open(profile_name).unwrap();
        let result = remote_profile.install_profile();
        dbg!(&result);
        assert!(&result.is_ok());
        let local_profile = result.unwrap();
        dbg!(&local_profile);

    }
    #[test]
    #[serial]
    fn test_delete_profile(){
        let profile_name = "new_profile";
        let start_profile = RemoteProfile::open(profile_name).unwrap();
        let delete_profile = start_profile.copy("delete_profile").unwrap();

        let mut file_names:Vec<String> = Vec::new();
        for x in sftp_list_dir(PathBuf::from(SFTP_PROFILES_DIR).as_path()).unwrap(){
            file_names.push(x.0.file_name().unwrap().to_str().unwrap().to_string());
        }
        assert!(file_names.contains(&delete_profile.name));
        delete_profile.delete().unwrap();
        let mut file_names:Vec<String> = Vec::new();
        for x in sftp_list_dir(PathBuf::from(SFTP_PROFILES_DIR).as_path()).unwrap(){
            file_names.push(x.0.file_name().unwrap().to_str().unwrap().to_string());
        }
        assert!(!file_names.contains(&"delete_me".to_string()));
    }

    #[test]
    fn test_install_mods(){
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let profile_name = "new_profile";
        let remote_profile = RemoteProfile::open(profile_name).unwrap();
        let _ = fs::remove_dir_all(base_path.join("profiles").join(profile_name).join("mods"));
        let _ = fs::create_dir(base_path.join("profiles").join(profile_name).join("mods"));
        let result = remote_profile.install_addons(AddonType::Mod);
        assert!(result.is_ok());
        let local_profile = LocalProfile::open(profile_name).unwrap();
        println!("{:?}",local_profile);
        assert!(&local_profile.mods.unwrap().len() > &0);

    }
    #[test]
    fn test_read_resource_packs(){
        let profile_name = "jman_modpack";
        let remote_profile = RemoteProfile::open(profile_name).unwrap();
        dbg!(&remote_profile);
    }
    #[test]
    fn test_read_specific_remote_profile(){
        let result = RemoteProfile::open("new_profile");
        assert!(result.is_ok());
    }

}
