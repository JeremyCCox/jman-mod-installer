use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ssh2::{Sftp};
use crate::installer::{InstallerConfig, InstallerError};
const SFTP_MODS_PATH:&str = "/upload/mods";
const SFTP_RESOURCE_PACKS_PATH: &str ="/upload/resource_packs";

#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub enum AddonType{
    ResourcePack,
    Mod,
}
impl AddonType{
    pub fn get_remote_dir(&self)->PathBuf{
        return match self{
            AddonType::ResourcePack => {
                PathBuf::from(SFTP_RESOURCE_PACKS_PATH)
            }
            AddonType::Mod => {
                PathBuf::from(SFTP_MODS_PATH)
            }
        }
    }
    pub fn get_local_dir(&self,profile_name:&str)->Result<PathBuf,InstallerError>{
        let def_path = InstallerConfig::open()?.default_game_dir.unwrap();
        return match self{
            AddonType::ResourcePack => {
                Ok(def_path.join("profiles").join(profile_name).join("resourcepacks"))
            }
            AddonType::Mod => {
                Ok(def_path.join("profiles").join(profile_name).join("mods"))
            }
        }
    }
}
pub struct AddonManager{

}

impl AddonManager{
    pub fn read_remote_addon(addon_type:AddonType)->Result<Vec<ProfileAddon>,InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        let remote_path = addon_type.get_remote_dir();
        let mut packs:Vec<ProfileAddon> = Vec::new();
        let val = sftp.readdir(remote_path.as_path())?;
        for x in val {
            if x.1.is_dir(){
                packs.push(ProfileAddon::open_remote(x.0.file_name().unwrap().to_str().unwrap(), AddonType::ResourcePack)?)
            }
        }
        Ok(packs)
    }
}


#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAddon{
    pub addon_type:AddonType,
    pub name:String,
    pub file_name:String,
    pub location:PathBuf,
    pub versions:Vec<String>,
    pub dependencies:Vec<String>
}
impl ProfileAddon{
    pub fn new(filename:&str,addon_type:AddonType)->Self{
        let v:Vec<&str> = filename.split(".").collect::<Vec<&str>>();
        let mut clean_name:Vec<&str> = Vec::new();
        for x in v {
            match x.eq("jar"){
                true => {
                    break
                }
                false => {
                    clean_name.push(x);
                }
            }
        }
        Self{
            addon_type,
            name: clean_name.join("."),
            file_name: filename.into(),
            location: Default::default(),
            versions:vec![],
            dependencies: vec![],
        }
    }

    pub fn open_remote(name: &str,addon_type: AddonType) -> Result<Self, InstallerError>
    where
        Self: Sized
    {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_pack = addon_type.get_remote_dir().join(name).join("pack.json");
        match sftp.open(remote_pack.as_path()){
            Ok(file) => {
                Ok(serde_json::from_reader(file)?)
            }
            Err(err) => {
                Err(InstallerError::Ssh2(err))
            }
        }
    }

    pub fn open_local(name: &str,addon_type:AddonType) -> Result<Self, InstallerError>
    where
        Self: Sized
    {
        let rp = Self::new(&name,addon_type.clone());
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_pack = addon_type.get_remote_dir().join(&rp.name).join("pack.json");
        match sftp.open(remote_pack.as_path()){
            Ok(file) => {
                Ok(serde_json::from_reader(file).unwrap_or_else(|_| rp))
            }
            Err(_) => {
                Ok(rp)
            }
        }
    }

    pub fn upload(&self, source: &PathBuf) -> Result<(), InstallerError> {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let pack_dir= &self.addon_type.get_remote_dir().join(&self.name);
        match sftp.readdir(pack_dir){
            Ok(_) => {
                self.update_addon_pack(pack_dir,&sftp)?;
            }
            Err(_) => {
                _ = sftp.mkdir(pack_dir.as_path(),1002);
                self.upload_addon(source,&pack_dir,&sftp)?;
                self.update_addon_pack(pack_dir,&sftp)?;
            }
        }
        Ok(())
    }
    pub fn update_addon_pack(&self,dest :&PathBuf,sftp:&Sftp)->Result<(),InstallerError>{
        let mut file = sftp.create(dest.join("pack.json").as_path())?;
        let self_json = serde_json::to_string_pretty(&self)?;
        file.write(self_json.as_ref())?;
        Ok(())
    }
    pub fn upload_addon(&self,source:&PathBuf,dest:&PathBuf,sftp:&Sftp) ->Result<(),InstallerError>{
        let mut upload_file = fs::File::open(source.join(&self.file_name))?;
        let mut remote_file = sftp.create(dest.join(&self.file_name).as_path())?;
        io::copy(&mut upload_file, &mut remote_file)?;
        Ok(())
    }

    pub fn download(&self, location: &PathBuf) -> Result<(), InstallerError> {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_dir= self.addon_type.get_remote_dir().join(&self.name);
        _ = fs::create_dir_all(&location);
        let mut local_file = fs::File::create(&location.join(&self.file_name))?;
        let mut remote_file = sftp.open(remote_dir.join(&self.file_name).as_path())?;
        io::copy(&mut remote_file, &mut local_file)?;
        Ok(())
    }
    pub fn update_remote(&self)->Result<(),InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect()?;
        let pack_dir= &self.addon_type.get_remote_dir().join(&self.name);
        self.update_addon_pack(pack_dir,&sftp)

    }
}

#[cfg(test)]
mod tests{
    use std::fs::File;
    use std::path::PathBuf;
    use serial_test::serial;
    use crate::installer::InstallerConfig;
    use crate::addons::{AddonManager, AddonType, ProfileAddon};

    #[test]
    fn test_new_mod(){
        let rp = ProfileAddon::new("optifine.jar",AddonType::Mod);
        assert_eq!(rp.name,"optifine");
        assert_eq!(rp.file_name,"optifine.jar")
    }
    #[test]
    #[serial]
    fn test_open_remote_mod(){
        let result = ProfileAddon::open_remote("optifine",AddonType::Mod);
        dbg!(&result);
        assert!(result.is_ok());
        let rp = result.unwrap();
        assert_eq!(rp.file_name,"optifine.jar")
    }
    #[test]
    fn test_open_local_mod(){
        let exists_result = ProfileAddon::open_local("optifine.jar",AddonType::Mod);
        assert!(exists_result.is_ok());
        let erp = exists_result.unwrap();
        assert_eq!(erp.file_name,"optifine.jar");
        assert_eq!(erp.name,"optifine");

        let ne_result = ProfileAddon::open_local("This does not exist.jar",AddonType::Mod);
        assert!(ne_result.is_ok());
        let nerp = ne_result.unwrap();
        assert_eq!(nerp.file_name,"This does not exist.jar");
        assert_eq!(nerp.name,"This does not exist")

    }
    // #[test]
    // fn test_read_remote_packs(){
    //     let result = ModsManager::read_remote_mods();
    //     dbg!(&result);
    //     assert!(result.is_ok());
    //     let remote_paths = result.unwrap();
    //     dbg!(remote_paths);
    // }
    #[test]
    fn test_upload_mod(){
        let installer_config = InstallerConfig::open().unwrap();
        let source = PathBuf::from(installer_config.default_game_dir.unwrap().join("profiles").join("new_profile"));
        File::create(source.join("mods").join("optifine.jar")).expect("Could not create mod");
        let rp = ProfileAddon::new("optifine.jar",AddonType::Mod);
        dbg!(&rp);
        let result = rp.upload(&source);
        dbg!(&result);
        assert!(result.is_ok());
    }
    #[test]
    fn test_download_mod(){
        let installer_config = InstallerConfig::open().unwrap();
        let profile_path = PathBuf::from(installer_config.default_game_dir.unwrap().join("profiles").join("new_profile"));
        let rp = ProfileAddon::new("optifine.jar",AddonType::Mod);
        let result = rp.download(&profile_path);
        assert!(result.is_ok());

    }
}