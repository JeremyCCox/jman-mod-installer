use std::{env, fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write};
use std::net::TcpStream;
use std::path::{PathBuf};
use std::process::Command;
use chrono::{Utc};
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};
use crate::sftp::{copy_dir_all, InstallerError, RemoteProfile, sftp_list_dir};

const SFTP_PROFILES_DIR: &str = "/upload/profiles/";

pub fn list_profiles_mods(profile_path:&PathBuf) -> Result<Vec<PathBuf>,InstallerError> {
    let mods = fs::read_dir(profile_path.join("mods").as_path()).expect("Could not read dir!");
    let mut mod_names = Vec::new();
    for x in mods {
        let entry = x.unwrap();
        let val = entry.path();
        mod_names.push(val);
    };
    Ok(mod_names)
}
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ProfileMod{
    name:String,
    version:Option<String>,
    required:Option<bool>,
    enabled:Option<bool>
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSettings{
    crash_assistance : bool,
    enable_advanced : bool,
    enable_analytics  : bool,
    enable_historical : bool,
    enable_releases : bool,
    enable_snapshots : bool,
    keep_launcher_open : bool,
    profile_sorting : String,
    show_game_log : bool,
    show_menu : bool,
    sound_on : bool
}
#[derive(Serialize,Deserialize,Debug)]
pub struct LauncherProfiles{
    pub profiles: HashMap<String, LauncherProfile>,
    pub settings: LauncherSettings,
    pub version: u64,
}
impl LauncherProfiles{
    pub fn open() ->Self{
        let base_path =  InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let file = File::open(base_path.join("launcher_profiles.json")).expect("Could not open launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles.json"),base_path.join("launcher_profiles-copy.json")).expect("Could not store launcher_profiles.json into launcher_profiles.json");
        let launcher_profiles: LauncherProfiles = serde_json::from_reader(&file).expect("Could not read launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles-copy.json"),base_path.join("launcher_profiles.json")).expect("Could not restore launcher_profiles.json from copy ");
        launcher_profiles
    }
    pub fn from_file(base_path: &PathBuf) ->Self{
        let file = File::open(base_path.join("launcher_profiles.json")).expect("Could not open launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles.json"),base_path.join("launcher_profiles-copy.json")).expect("Could not store launcher_profiles.json into launcher_profiles.json");
        let launcher_profiles: LauncherProfiles = serde_json::from_reader(&file).expect("Could not read launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles-copy.json"),base_path.join("launcher_profiles.json")).expect("Could not restore launcher_profiles.json from copy ");
        launcher_profiles
    }
    pub fn insert_profile(&mut self,mut launcher_profile:LauncherProfile, profile_name:&str)->Result<(),InstallerError>{
        let base_path = &InstallerConfig::open()?.default_game_dir.unwrap();
        launcher_profile.name=Some(profile_name.to_string());
        launcher_profile.game_dir = Some(base_path.join("profiles").join(&profile_name));
        launcher_profile.created = Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        self.profiles.insert((&profile_name).parse().unwrap(), launcher_profile);
        self.save();
        Ok(())
    }
    pub fn remove_profile(&mut self, profile_name:&str) ->Result<(),InstallerError>{
        self.profiles.remove(&profile_name.to_string());
        self.save();
        Ok(())
    }
    pub fn save(&self){
        let base_path = InstallerConfig::open().unwrap().default_game_dir.unwrap();
        let launcher_profiles_json = serde_json::to_string_pretty(&self).unwrap();
        fs::rename(base_path.join("launcher_profiles.json"),base_path.join("launcher_profiles-copy.json")).expect("Could not restore launcher_profiles.json from copy ");
        match File::create(base_path.join("launcher_profiles.json")){
            Ok(mut new_file) => {
                new_file.write((&launcher_profiles_json).as_ref()).expect("Could not write new launcher_profiles.json");
                println!("File updated?");
                match fs::metadata(base_path.join("launcher_profiles-copy.json")){
                    Ok(_)=>{
                        fs::remove_file(base_path.join("launcher_profiles-copy.json")).expect("Could not cleanup old launcher_profiles-copy");
                    }
                    Err(_)=>{}
                }
            }
            Err(_) => {
                fs::rename(base_path.join("launcher_profiles-copy.json"),base_path.join("launcher_profiles.json")).expect("Could not restore launcher_profiles.json from copy ");
            }
        };
    }
}
#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherProfile {
    // #[serde(with = "ts_seconds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created :Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_dir : Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon : Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_version_id : Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name : Option<String>,
    // profile_type : String, // Should be type but type is reserved in rust!
}
impl LauncherProfile{
    // pub fn new() -> Self{
    //     Self{
    //         created: Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
    //         game_dir: None,
    //         icon: Some("Enchanting_Table".parse().unwrap()),
    //         last_version_id:  Some("fabric-loader-0.15.11-1.20.1".parse().unwrap()),
    //         name: None,
    //     }
    // }
    pub fn new(name: &str) -> Self{
        Self{
            name:Some(name.parse().unwrap()),
            ..Self::default()
        }
    }
    pub fn from_file(profile_name:&str)->Result<Self,InstallerError>{
        let installer_config = InstallerConfig::open()?;
        let base_path = PathBuf::from(installer_config.default_game_dir.unwrap());
        let local_launcher_profiles = File::open(&base_path.join("launcher_profiles.json")).unwrap();
        let json:serde_json::Value = serde_json::from_reader(&local_launcher_profiles).expect("Could not read JSON from file");
        let mut launcher_profile:LauncherProfile = LauncherProfile::new(profile_name);
        for (key,value) in json["profiles"].as_object().unwrap(){
            match value["gameDir"].as_str(){
                None=>{},
                Some(x) if PathBuf::from(x).eq(&base_path.join("profiles").join(profile_name)) => {
                    let obj = value["lastVersionId"].as_str().unwrap();
                    launcher_profile.last_version_id = Some(obj.to_string());
                    println!("{} {}",key,obj)
                },
                Some(_)=>{}
            }
        };
        Ok(launcher_profile)
    }
    // pub fn save_file(&self, profile_path:&PathBuf)->Result<(),InstallerError>{
    //     let launcher_json = serde_json::to_string(self).unwrap();
    //     let mut launcher_file = File::create(&profile_path.join("launcher_profile.json")).unwrap();
    //     launcher_file.write(launcher_json.as_ref()).expect("TODO: panic message");
    //     Ok(())
    // }
}
impl Default for LauncherProfile{
    fn default() -> Self {
        Self{
            created:  Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            game_dir: None,
            icon: Some("Enchanting_Table".parse().unwrap()),
            last_version_id: Some("fabric-loader-0.15.11-1.20.1".parse().unwrap()),
            name: Some("profile_name".parse().unwrap()),
        }
    }
}
pub enum GameProfile{
    Local(LocalProfile),
    Remote(RemoteProfile),
}
impl From<LocalProfile> for GameProfile{
    fn from(value: LocalProfile) -> Self {
        GameProfile::Local(value)
    }
}
impl From<RemoteProfile> for GameProfile{
    fn from(value: RemoteProfile) -> Self {
        GameProfile::Remote(value)
    }
}
impl From<&RemoteProfile> for LocalProfile{
    fn from(value: &RemoteProfile) -> LocalProfile {
        let mut local = LocalProfile::new(&value.name);
        local.scaffold().expect("Could not scaffold local profile");
        local.launcher_profile=value.launcher_profile.clone();
        LauncherProfiles::open().insert_profile(local.launcher_profile.clone().unwrap(),&value.name).expect("Could not insert Launcher Profile");
        local
    }
}
pub trait Profile{
    fn new(profile_name:&str)->Self;
    fn create (profile_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn scaffold(&self) ->Result<(),InstallerError>;
    fn open(profile_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn copy (self,copy_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn delete(self)->Result<(),InstallerError>;
    fn read_mods(&mut self)->Result<(),InstallerError>;
    fn write_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn read_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn rename_profile(&mut self,new_name:&str)->Result<(),InstallerError>;

}

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



#[derive(Serialize,Deserialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallerConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_game_dir:Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sftp_server:Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sftp_port:Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sftp_username:Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sftp_password:Option<String>,
}
impl Default for InstallerConfig{
    fn default() -> Self {
        Self{
            default_game_dir: None,
            sftp_server: None,
            sftp_port: None,
            sftp_username: None,
            sftp_password: None,
        }
    }
}

impl InstallerConfig{
    // pub fn new()->Self{
    //     Self::default()
    // }

    #[cfg(test)]
    pub fn test_new()->Self{
        Self{
            default_game_dir: Some(PathBuf::from("test\\.minecraft")),
            sftp_server: Some("bigbrainedgamers.com".parse().unwrap()),
            sftp_port: Some("2222".parse().unwrap()),
            sftp_username: Some("headless".parse().unwrap()),
            sftp_password: Some("pword".parse().unwrap()),
        }
    }
    #[cfg(test)]
    pub fn test_open()->Result<Self,InstallerError>{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        match File::open(app_dir.join("test-config.json")){
            Ok(file) => {
                println!("{:?}",file);
                let read_config:InstallerConfig = serde_json::from_reader(file).expect("Could not read from config.json");
                println!("{:?}",read_config);
                Ok(read_config)
            },
            Err(err) => {
                Err(InstallerError::from(err))
            }
        }
    }
    #[cfg(test)]
    pub fn test_save(&self)->Result<(),InstallerError>{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        let _ = fs::create_dir(&app_dir);
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(&app_dir.join("test-config.json")).expect("Could not create config.json");
        file.write(json.as_ref()).expect("Could not save config.json");
        Ok(())
    }
    // pub fn from_game_dir(game_dir:&str)->Self{
    //     Self{
    //         default_game_dir:Some(game_dir.to_string()),
    //         ..Self::default()
    //     }
    // }
    pub fn save_config(&self)->Result<(),InstallerError>{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        let _ = fs::create_dir(&app_dir);
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(&app_dir.join("config.json")).expect("Could not create config.json");
        file.write(json.as_ref()).expect("Could not save config.json");
        Ok(())
    }
    pub fn open()->Result<Self,InstallerError>{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        match File::open(app_dir.join("config.json")){
            Ok(file) => {
                let read_config:InstallerConfig = serde_json::from_reader(file)?;
                Ok(read_config)
            },
            Err(err) => {
                Err(InstallerError::from(err))
            }
        }
    }
    pub fn sftp_connect(&self)->Result<Sftp,InstallerError>{
        let address = format!("{}:{}",&self.sftp_server.clone().unwrap(),&self.sftp_port.clone().unwrap());
        let tcp = TcpStream::connect(address)?;

        let mut sess = Session::new().expect("Could not open session!");

        sess.set_tcp_stream(tcp);

        for _try in 1..=4{
            match sess.handshake(){
                Ok(()) => {
                    break
                }
                Err(error) => {
                    if _try < 4{
                        continue
                    }else{
                        return Err(InstallerError::from(error))
                    }
                }
            }
        }
        sess.userauth_password(&self.sftp_username.clone().unwrap(), &self.sftp_password.clone().unwrap()).expect("Auth failed");
        match sess.sftp() {
            Ok(sftp) => Ok(sftp),
            Err(error) => Err(InstallerError::from(error))
        }
    }
    pub fn sftp_safe_connect(&self)->Result<Sftp,InstallerError>{
        let mut i:usize  = 0;
        return loop{
            match self.sftp_connect() {
                Ok(sftp) => {
                    break Ok(sftp)
                }
                Err(err) => {
                    i += 1;
                    if i == 4{
                        break Err(err)
                    }
                }
            }
        }
    }
    pub fn clear()->Result<(),InstallerError> {
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        let _ = fs::remove_file(app_dir.join("config.json"));
        let _ = fs::remove_file(app_dir.join("test-config.json"));


        Ok(fs::remove_file(app_dir.join("config.json")).unwrap())
    }
}
pub fn create_profile(base_path:&PathBuf,profile_name:&str)-> Result<(),InstallerError>{
    let installer_config = InstallerConfig::open().unwrap();
    println!("{:?}",installer_config);
    let profile_path = PathBuf::from(installer_config.default_game_dir.unwrap()).join("profiles").join(profile_name);
    let launcher_profile = LauncherProfile::new(profile_name);
    fs::create_dir_all(&profile_path.join("mods")).expect("Couldnt create the profile directory");
    fs::copy(&base_path.join("options.txt"),&profile_path.join("options.txt")).expect("Could not create options copy");
    fs::copy(&base_path.join("servers.dat"),&profile_path.join("servers.dat")).expect("Could not create options copy");
    let mut launcher_profiles = LauncherProfiles::from_file(base_path);
    launcher_profiles.insert_profile(launcher_profile,profile_name)?;
    Ok(())
}

pub fn create_mods_folder(base_path:&PathBuf,profile_name:&str)-> Result<(),InstallerError >{
    let mods_path = base_path.join("profiles").join(profile_name).join("mods");
    match fs::metadata(&mods_path){
        Ok(_) => return Ok(()),
        Err(_) => {
            Ok(fs::create_dir_all(&mods_path)?)
        }
    }
}
// pub fn read_launcher_profiles(base_path:&PathBuf)->Result<Option<&Map<&String,&Value>>,InstallerError>{
//     Ok(Some(json["profiles"].as_object()))
// }
// pub fn install_launcher_profile(base_path:&PathBuf,profile_name:&str)->Result<(),Error>{
//     let launcher_profile = LauncherProfile::from_file( profile_name);
//     // let mut LauncherProfiles = LauncherProfiles::from_file(base_path);
//     launcher_profile.save_file(&base_path);
//     Ok(())
// }
// pub fn create_launcher_profile(base_path:&PathBuf,profile_path:&PathBuf,profile_name:&str)->Result<(),InstallerError>{
//     let launcher_profile = LauncherProfile::from_file();
//     Ok(())
// }
pub fn open_profile_location(profile_name:&str)->Result<(),InstallerError>{
    let profile_path = InstallerConfig::open()?.default_game_dir.unwrap().join("profiles").join(profile_name);
    match env::consts::OS{
        x if x.eq("windows")=>{
            Command::new("explorer").arg(profile_path).spawn().unwrap();
        },
        x if x == "linux" => {
            Command::new("xdg-open").arg(profile_path).spawn().unwrap();
        },
        x if x == "macos" => {
            Command::new("open").arg(profile_path).spawn().unwrap();
        },
        _ => {
            panic!("Unrecognized Operating system!")
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests{
    use serial_test::serial;
    use crate::sftp::sftp_upload_profile;
    use super::*;
    const BASE_PATH_STRING: &str = "test\\.minecraft";
    const SFTP_PROFILES_DIR: &str = "/upload/profiles/";
    // #[test]
    pub fn setup_test_profile()->Result<(),InstallerError>{
        let installer_config:InstallerConfig = InstallerConfig::test_new();
        installer_config.test_save()
    }
    // #[test]
    // fn test_insert_profile(){
    //     let mut launcher_profiles = LauncherProfiles::from_file(&PathBuf::from(BASE_PATH_STRING));
    //     let launcher_profile = LauncherProfile::new("test_profile");
    //     launcher_profiles.insert_profile(launcher_profile,"test_profile").expect("Could not insert profile!");
    //
    // }

    #[test]
    fn test_new_local_profile(){
        let profile_name = "new_profile";
        let new_profile = LocalProfile::new(profile_name);
        assert_eq!(new_profile.name, profile_name)
    }
    #[test]
    fn test_upload_profile(){
        let profile_name = "new_profile";

        // SFTP client for tests
        let sftp =  InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();

        // Test that the function ran without errors
        assert!(sftp_upload_profile(profile_name).is_ok());

        // Test that the directory was created properly
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").as_path()).unwrap().is_dir());

        // Test that the launcher_profile was created
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").join("launcher_profile.json").as_path()).unwrap().is_file());

        // Test that the mods folder exists and contains testjar.jar
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").join("mods").join("testjar.jar").as_path()).unwrap().is_file())

    }
    #[test]
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
    fn list_mods(){
        let mods_path = PathBuf::from("test").join(".minecraft").join("profiles").join("test_profile");
        let mods = list_profiles_mods(&mods_path).unwrap();
        assert_eq!(mods,[mods_path.join("mods").join("testjar.jar")])
    }
    #[test]
    #[serial]
    fn test_create_profile(){
        let profile_name = "create_profile";
        let new_profile = LocalProfile::create(profile_name).unwrap();
        println!("{:?}",new_profile);
    }
    #[test]
    fn test_connect_profile(){
        let installer_config = InstallerConfig::test_new();
        let sftp_result = installer_config.sftp_connect();
        assert!(sftp_result.is_ok());
    }

    // #[test]
    // fn test_create_launcher_profile(){
    //     let base_path = PathBuf::from(BASE_PATH_STRING);
    //     let profile_name = "new_profile";
    //     let profile_path = base_path.join("profiles").join(profile_name);
    //     create_launcher_profile(&base_path,&profile_path, profile_name).expect("Could not create new Profile!");
    //     // let meta_data = fs::metadata(base_path.join("profiles").join(profile_name)).unwrap();
    //     // assert!(meta_data.is_dir())
    //     // assert!()
    // }
    #[test]
    fn test_mods_folder(){
        let base_path = PathBuf::from(BASE_PATH_STRING);
        assert!(create_mods_folder(&base_path,"new_mods_folder").is_ok());
        assert!(create_mods_folder(&base_path,"new_profile").is_ok());
        fs::remove_dir_all(base_path.join("profiles/new_mods_folder")).unwrap();
    }

    #[test]
    #[serial]
    fn test_create_config(){
        let installer_config = InstallerConfig::test_new();
        assert!(installer_config.test_save().is_ok());
        setup_test_profile().unwrap()
    }
    #[test]
    #[serial]
    fn test_read_config(){
        setup_test_profile().unwrap();
        let installer_config = InstallerConfig::test_open().unwrap();
        assert!(installer_config.default_game_dir.eq(&Some("test\\.minecraft".parse().unwrap())));
    }

    #[test]
    fn test_local_profile_read_launcher(){
        let mut local = LocalProfile::new("new_profile");
        let result = local.read_launcher_profile();
        assert!(result.is_ok())
    }
    #[test]
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