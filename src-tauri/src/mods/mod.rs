use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::installer::{InstallerConfig, InstallerError};
use crate::profiles::ProfileAddon;
use crate::resource_packs::ResourcePack;

const SFTP_MODS_PATH:&str = "/upload/mods";
const SFTP_RESOURCE_PACKS_PATH: &str ="/upload/resource_packs";

pub struct AddonManager{

}
impl AddonManager{
    pub fn read_remote_mods()->Result<Vec<Mod>,InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        let remote_path = PathBuf::from(SFTP_MODS_PATH);
        let mut packs:Vec<Mod> = Vec::new();
        let val = sftp.readdir(remote_path.as_path())?;
        for x in val {
            if x.1.is_dir(){
                packs.push(Mod::open_remote(x.0.file_name().unwrap().to_str().unwrap())?)
            }
        }
        Ok(packs)
    }
    pub fn read_remote_packs()->Result<Vec<ResourcePack>,InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        let remote_path = PathBuf::from(SFTP_RESOURCE_PACKS_PATH);
        let mut packs:Vec<ResourcePack> = Vec::new();
        let val = sftp.readdir(remote_path.as_path())?;
        for x in val {
            if x.1.is_dir(){
                packs.push(ResourcePack::open_remote(x.0.file_name().unwrap().to_str().unwrap())?)
            }
        }
        Ok(packs)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod{
    pub name:String,
    pub file_name:String,
    pub location:PathBuf,
    pub versions:Vec<String>,
    pub dependencies:Vec<String>
}
impl ProfileAddon for Mod{
    fn new(filename:&str)->Self{
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
            name: clean_name.join("."),
            file_name: filename.into(),
            location: Default::default(),
            versions:vec![],
            dependencies: vec![],
        }
    }

    fn open_remote(name: &str) -> Result<Self, InstallerError>
    where
        Self: Sized
    {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_pack = PathBuf::from(SFTP_MODS_PATH).join(name).join("pack.json");
        match sftp.open(remote_pack.as_path()){
            Ok(file) => {
                Ok(serde_json::from_reader(file)?)
            }
            Err(err) => {
                Err(InstallerError::Ssh2(err))
            }
        }
    }

    fn open_local(name: &str) -> Result<Self, InstallerError>
    where
        Self: Sized
    {
        let rp = Self::new(&name);
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_pack = PathBuf::from(SFTP_MODS_PATH).join(&rp.name).join("pack.json");
        match sftp.open(remote_pack.as_path()){
            Ok(file) => {
                Ok(serde_json::from_reader(file).unwrap_or_else(|err| rp))
            }
            Err(_) => {
                Ok(rp)
            }
        }
    }

    fn upload(&self, source: &PathBuf) -> Result<(), InstallerError> {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let pack_dir= PathBuf::from(SFTP_MODS_PATH).join(&self.name);
        _ = sftp.mkdir(pack_dir.as_path(),1002);
        let mut upload_file = fs::File::open(source.join("mods").join(&self.file_name))?;
        let mut remote_file = sftp.create(pack_dir.join(&self.file_name).as_path())?;

        let mut file = sftp.create(pack_dir.join("pack.json").as_path())?;
        let self_json = serde_json::to_string_pretty(&self)?;
        file.write(self_json.as_ref())?;
        io::copy(&mut upload_file, &mut remote_file)?;
        Ok(())
    }

    fn download(&self, location: &PathBuf) -> Result<(), InstallerError> {
        let installer_config= InstallerConfig::open()?;
        let sftp = installer_config.sftp_safe_connect()?;
        let remote_dir= PathBuf::from(SFTP_MODS_PATH).join(&self.name);
        let pack_dir = location.join("mods");
        _ = fs::create_dir_all(&pack_dir);
        let mut local_file = fs::File::create(&pack_dir.join(&self.file_name))?;
        let mut remote_file = sftp.open(remote_dir.join(&self.file_name).as_path())?;
        io::copy(&mut remote_file, &mut local_file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use std::path::PathBuf;
    use serial_test::serial;
    use crate::installer::InstallerConfig;
    use crate::mods::{Mod, SFTP_MODS_PATH};
    use crate::profiles::{Profile, ProfileAddon};
    use crate::sftp::sftp_list_dir;

    #[test]
    fn test_new_mod(){
        let rp = Mod::new("optifine.jar");
        assert_eq!(rp.name,"optifine");
        assert_eq!(rp.file_name,"optifine.jar")
    }
    #[test]
    #[serial]
    fn test_open_remote_mod(){
        let result = Mod::open_remote("optifine");
        dbg!(&result);
        assert!(result.is_ok());
        let rp = result.unwrap();
        assert_eq!(rp.file_name,"optifine.jar")
    }
    #[test]
    fn test_open_local_mod(){
        let exists_result = Mod::open_local("optifine.jar");
        assert!(exists_result.is_ok());
        let erp = exists_result.unwrap();
        assert_eq!(erp.file_name,"optifine.jar");
        assert_eq!(erp.name,"optifine");

        let ne_result = Mod::open_local("This does not exist.jar");
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
        let mut rp = Mod::new("optifine.jar");
        let result = rp.upload(&source);
        dbg!(&result);
        assert!(result.is_ok());
        sftp_list_dir(PathBuf::from(SFTP_MODS_PATH).join(&rp.name).as_path()).unwrap();
    }
    #[test]
    fn test_download_mod(){
        let installer_config = InstallerConfig::open().unwrap();
        let location = PathBuf::from(installer_config.default_game_dir.unwrap().join("profiles").join("new_profile"));
        let mut rp = Mod::new("optifine.jar");
        let result = rp.download(&location);
        println!("{:?}",result);
        assert!(result.is_ok());

    }
}