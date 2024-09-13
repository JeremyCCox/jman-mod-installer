use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::{ PathBuf};
use serde::{Deserialize, Serialize};
use ssh2::{Sftp};
use crate::installer::{InstallerConfig, InstallerError};
use crate::sftp::sftp_remove_dir;
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
    pub fn get_addon_manifest(&self)->PathBuf{
        return match self {
            AddonType::ResourcePack => {
                tauri::api::path::data_dir().unwrap().join("jman-mod-installer").join("remote-resourcepacks.json")
            }
            AddonType::Mod => {
                tauri::api::path::data_dir().unwrap().join("jman-mod-installer").join("remote-mods.json")
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

    pub fn read_remote_addon(addon_name:&str,addon_type:AddonType)->Result<ProfileAddon,InstallerError>{
        Ok(AddonManager::read_addon_manifest(addon_type)?.into_iter().find(|addon| addon.addon_matches_name(addon_name)).unwrap())
    }
    pub fn read_remote_addons(addon_type:AddonType) ->Result<Vec<ProfileAddon>,InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
        let remote_path = addon_type.get_remote_dir();
        let mut packs:Vec<ProfileAddon> = Vec::new();
        let val = sftp.readdir(remote_path.as_path())?;
        for x in val {
            if x.1.is_dir(){
                packs.push(ProfileAddon::open_remote(x.0.file_name().unwrap().to_str().unwrap(),addon_type)?)
            }
        }
        Ok(packs)
    }
    pub fn update_addon(addon:ProfileAddon)->Result<(),InstallerError>{
        addon.update_remote()?;
        let mut manifest_list = AddonManager::read_addon_manifest(addon.addon_type)?.to_owned();
        manifest_list.retain(|manifest_addon| !manifest_addon.addon_matches(&addon));
        manifest_list.push(addon.clone());
        AddonManager::write_addon_manifest(&manifest_list,addon.addon_type)?;
        Ok(())
    }
    pub fn update_addon_manifest(addon_type: AddonType)->Result<(),InstallerError>{
        AddonManager::write_addon_manifest(&AddonManager::read_remote_addons(addon_type)?,addon_type)
    }
    pub fn write_addon_manifest(addons:&Vec<ProfileAddon>,addon_type: AddonType)->Result<(),InstallerError>{
        let mut manifest = File::create(addon_type.get_addon_manifest())?;
        manifest.write(serde_json::to_string_pretty(&addons)?.as_ref())?;
        Ok(())
    }
    pub fn read_addon_manifest(addon_type: AddonType)->Result<Vec<ProfileAddon>,InstallerError>{
        match File::open(addon_type.get_addon_manifest()){
            Ok(manifest) =>         Ok(serde_json::from_reader(manifest)?),
            Err(_) => {
                let addons = AddonManager::read_remote_addons(addon_type)?;
                AddonManager::write_addon_manifest(&addons,addon_type)?;
                Ok(addons)
            }
        }
    }
    pub fn add_new_addons(addons:Vec<ProfileAddon>,addon_type: AddonType)->Result<(),InstallerError>{
        AddonManager::insert_addons_into_manifest(addons.clone(),addon_type)?;
        for addon in addons {
            addon.upload(&addon.location).unwrap()
        }
        Ok(())
    }
    pub fn insert_addons_into_manifest(addon_list:Vec<ProfileAddon>, addon_type:AddonType) ->Result<(),InstallerError>{
        let mut unique_list:Vec<ProfileAddon> = Vec::new();
        let manifest_list = AddonManager::read_addon_manifest(addon_type)?;
        for x in addon_list {
            if !manifest_list.iter().any(|addon| addon.addon_matches(&x)){
                unique_list.push(x);
            }
        }
        let new_list = [unique_list,manifest_list].concat();
        AddonManager::write_addon_manifest(&new_list,addon_type)?;
        Ok(())
    }
    pub fn remove_addons_from_manifest(addon_list:Vec<ProfileAddon>, addon_type:AddonType) ->Result<(),InstallerError>{
        let mut manifest_list = AddonManager::read_addon_manifest(addon_type)?.to_owned();
        manifest_list.retain(|manifest_addon| !addon_list.iter().any(|target_addon| target_addon.addon_matches(manifest_addon)));
        AddonManager::write_addon_manifest(&manifest_list,addon_type)?;
        Ok(())
    }
    pub fn delete_addon(addon:ProfileAddon)->Result<(),InstallerError>{
        let addons:Vec<ProfileAddon> = Vec::from([addon.clone()]);
        let _ = addon.delete_remote();
        AddonManager::remove_addons_from_manifest(addons,addon.addon_type)
    }
    pub fn delete_addons_manifest(addon_type: AddonType)->Result<(),InstallerError>{
        Ok(fs::remove_file(addon_type.get_addon_manifest())?)
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
                match &self.file_name.as_str().eq(source.file_name().unwrap().to_str().unwrap()){
                    true => {
                        self.upload_addon(source,&pack_dir,&sftp)?;
                        self.update_addon_pack(pack_dir,&sftp)?;
                    },
                    false=>{
                        self.upload_addon(&source.join(&self.file_name),&pack_dir,&sftp)?;
                        self.update_addon_pack(pack_dir,&sftp)?;
                    }
                }

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
        let mut upload_file = fs::File::open(source)?;
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
    pub fn delete_remote(&self)->Result<(),InstallerError>{
        let sftp = InstallerConfig::open().unwrap().sftp_safe_connect()?;
        let pack_dir= &self.addon_type.get_remote_dir().join(&self.name);
        dbg!(&pack_dir);
        sftp_remove_dir(pack_dir,&sftp)
    }
    pub fn addon_matches(&self,addon:&Self)->bool{
        self.name == addon.name
    }
    pub fn addon_matches_name(&self,addon_name:&str)->bool{
        self.name == addon_name
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
    fn test_read_remote_addons(){
        let result = AddonManager::read_remote_addons(AddonType::ResourcePack);
        assert!(result.is_ok());
        let vec = result.unwrap();
        dbg!(vec);

    }
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
    fn test_upload_new_resourcepack(){
        let installer_config = InstallerConfig::open().unwrap();
        let source = PathBuf::from(installer_config.default_game_dir.unwrap().join("profiles").join("new_profile"));
        File::create(source.join("mods").join("optifine.jar")).expect("Could not create mod");
        let mut rp = ProfileAddon::new("colorful containers",AddonType::ResourcePack);
        rp.location = "C:\\Users\\Jeremy\\Downloads\\colourful containers.zip".parse().unwrap();
        rp.file_name= "colourful containers.zip".parse().unwrap();
        let result = rp.upload(&rp.location);
        dbg!(&result);
        assert!(result.is_ok());
    }
    #[test]
    fn test_download_mod(){
        let installer_config = InstallerConfig::open().unwrap();
        let rp = ProfileAddon::new("optifine.jar",AddonType::Mod);
        let result = rp.download(&rp.addon_type.get_local_dir("new_profile").unwrap());
        assert!(result.is_ok());
    }
    #[test]
    fn test_install_addon(){
        let installer_config = InstallerConfig::open().unwrap();
        let rp = ProfileAddon::new("optifine.jar",AddonType::Mod);
        let result = rp.download(&rp.addon_type.get_local_dir("new_profile").unwrap());
        assert!(result.is_ok());
    }
    #[test]
    #[serial]
    fn test_read_addon_manifest(){
        let result = AddonManager::read_addon_manifest(AddonType::ResourcePack);
        assert!(result.is_ok())
    }
    #[test]
    #[serial]
    fn test_write_addon_manifest(){
        let result = AddonManager::write_addon_manifest(&AddonManager::read_remote_addons(AddonType::ResourcePack).unwrap(), AddonType::ResourcePack);
        assert!(result.is_ok());
    }
    #[test]
    #[serial]
    fn test_insert_addon_manifest(){
        let addon_type = AddonType::ResourcePack;
        let test_addon = ProfileAddon::new("hee haa",addon_type);

        let result = AddonManager::insert_addons_into_manifest(Vec::from([ProfileAddon::new("hee haa", addon_type)]), addon_type);
        assert!(result.is_ok());

        let manifest = AddonManager::read_addon_manifest(addon_type).unwrap();
        assert!(&manifest.iter().any(|addon| addon.addon_matches(&test_addon)));

        AddonManager::remove_addons_from_manifest(Vec::from([ProfileAddon::new("hee haa", addon_type)]), addon_type).unwrap();
    }

    #[test]
    #[serial]
    fn test_remove_addon_manifest(){
        let addon_type = AddonType::ResourcePack;
        let test_addon = ProfileAddon::new("hee haa",addon_type);
        let _ = AddonManager::insert_addons_into_manifest(Vec::from([test_addon.clone()]), addon_type);
        let result = AddonManager::remove_addons_from_manifest(Vec::from([test_addon.clone()]), addon_type);
        assert!(&result.is_ok());
        let manifest = AddonManager::read_addon_manifest(addon_type).unwrap();
        assert!(!&manifest.iter().any(|addon| addon.addon_matches(&test_addon)))
    }
}