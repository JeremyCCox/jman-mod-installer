use std::{env, fs, io};
use std::collections::HashMap;
use std::fmt::{Error, Formatter};
use std::fs::File;
use std::io::{Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::read;
use chrono::{Utc};
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};
use log::error;


pub fn list_profiles_mods(profile_path:&PathBuf) -> Result<Vec<PathBuf>,io::Error> {
    let mods = fs::read_dir(profile_path.join("mods").as_path()).expect("Could not read dir!");
    let mut mod_names = Vec::new();
    for x in mods {
        let entry = x.unwrap();
        let val = entry.path();
        mod_names.push(val);
    };
    Ok(mod_names)
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

    pub fn from_file(base_path: &PathBuf) ->Self{
        let file = File::open(base_path.join("launcher_profiles.json")).expect("Could not open launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles.json"),base_path.join("launcher_profiles-copy.json")).expect("Could not store launcher_profiles.json into launcher_profiles.json");
        let mut launcher_profiles: LauncherProfiles = serde_json::from_reader(&file).expect("Could not read launcher_profiles.json");
        // fs::rename(base_path.join("launcher_profiles-copy.json"),base_path.join("launcher_profiles.json")).expect("Could not restore launcher_profiles.json from copy ");
        launcher_profiles
    }
    pub fn insert_profile(&mut self,mut launcher_profile:LauncherProfile, base_path:&PathBuf, profile_name:&str){
        launcher_profile.name=Some(profile_name.to_string());
        launcher_profile.game_dir = Some(base_path.join("profiles").join(&profile_name));
        launcher_profile.created = Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        self.profiles.insert((&profile_name).parse().unwrap(), launcher_profile);
        self.save(base_path)
    }
    pub fn save(&self, base_path:&PathBuf){
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
#[derive(Serialize,Deserialize,Debug)]
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
    pub fn new() -> Self{
        Self{
            created: Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            game_dir: None,
            icon: Some("Enchanting_Table".parse().unwrap()),
            last_version_id:  Some("fabric-loader-0.15.11-1.20.1".parse().unwrap()),
            name: None,
        }
    }
    pub fn from(name: &str) -> Self{
        Self{
            created:  Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            game_dir: None,
            icon: Some("Enchanting_Table".parse().unwrap()),
            last_version_id: Some("fabric-loader-0.15.11-1.20.1".parse().unwrap()),
            name: Some(name.to_string()),
        }
    }
    pub fn from_file(base_bath:&PathBuf,profile_name:&str)->Self{
        println!("{:?}",base_bath);
        let local_launcher_profiles = File::open(&base_bath.join("launcher_profiles.json")).unwrap();
        let json:serde_json::Value = serde_json::from_reader(&local_launcher_profiles).expect("Could not read JSON from file");
        let mut launcher_profile:LauncherProfile = LauncherProfile::from(profile_name);
        for (key,value) in json["profiles"].as_object().unwrap(){
            match value["gameDir"].as_str(){
                None=>{},
                Some(x) if PathBuf::from(x).eq(&base_bath.join("profiles").join(profile_name)) => {
                    let obj = value["lastVersionId"].as_str().unwrap();
                    launcher_profile.last_version_id = Some(obj.to_string());
                    println!("{} {}",key,obj)
                },
                Some(_)=>{}
            }
        };
        launcher_profile
    }
    pub fn save_file(&self, profile_path:&PathBuf){
        let launcher_json = serde_json::to_string(self).unwrap();
        let mut launcher_file = File::create(&profile_path.join("launcher_profile.json")).unwrap();
        launcher_file.write(launcher_json.as_ref()).expect("TODO: panic message");
    }

}

#[derive(Serialize,Deserialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallerConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_game_dir:Option<String>,
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
    pub fn new()->Self{
        Self::default()
    }

    #[cfg(test)]
    pub fn test_new()->Self{
        Self{
            default_game_dir: Some("test\\.minecraft".parse().unwrap()),
            sftp_server: Some("bigbrainedgamers.com".parse().unwrap()),
            sftp_port: Some("2222".parse().unwrap()),
            sftp_username: Some("headless".parse().unwrap()),
            sftp_password: Some("pword".parse().unwrap()),
        }
    }
    pub fn from_game_dir(game_dir:&str)->Self{
        Self{
            default_game_dir:Some(game_dir.to_string()),
            ..Self::default()
        }
    }
    pub fn save_config(&self)->Result<(),io::Error>{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        let _ = fs::create_dir(&app_dir);
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(&app_dir.join("config.json")).expect("Could not create config.json");
        file.write(json.as_ref()).expect("Could not save config.json");
        Ok(())
    }
    pub fn open()->Self{
        let app_dir = tauri::api::path::data_dir().unwrap().join("jman-mod-installer");
        match File::open(app_dir.join("config.json")){
            Ok(file) => {
                let read_config:InstallerConfig = serde_json::from_reader(file).expect("Could not read from config.json");
                read_config
            },
            Err(_) => {
                Self::default()
            }
        }
    }
    pub fn sftp_connect(&self)->Result<Sftp,ssh2::Error>{
        let address = format!("{}:{}",&self.sftp_server.clone().unwrap(),&self.sftp_port.clone().unwrap());
        let tcp = TcpStream::connect(address).expect("Could not establish tcp stream!");

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
                        return Err(error)
                    }
                }
            }
        }
        sess.userauth_password(&self.sftp_username.clone().unwrap(), &self.sftp_password.clone().unwrap()).expect("Auth failed");
        match sess.sftp() {
            Ok(sftp) => Ok(sftp),
            Err(error) => Err(error)
        }
    }
    pub fn sftp_safe_connect(&self)->Result<Sftp,ssh2::Error>{
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
}
pub fn create_profile(base_path:&PathBuf,profile_name:&str)-> Result<(),io::Error>{
    let profile_path = PathBuf::from(&base_path).join("profiles").join(profile_name);
    let launcher_profile = LauncherProfile::from(profile_name);
    fs::create_dir_all(&profile_path.join("mods")).expect("Couldnt create the profile directory");
    fs::copy(&base_path.join("options.txt"),&profile_path.join("options.txt")).expect("Could not create options copy");
    let mut launcher_profiles = LauncherProfiles::from_file(base_path);
    launcher_profiles.insert_profile(launcher_profile,base_path,profile_name);
    // let profile_config=  File::create(profile_path.join("profile_config.json")).unwrap();
    // let mut data = String::new();
    // launcher_profiles.read_to_string(& mut data).unwrap();

    // Create Launcher_profile.json to load into .minecraft/launcher_profiles on install

    Ok(())
}

pub fn create_mods_folder(base_path:&PathBuf,profile_name:&str)-> Result<(),io::Error >{
    let mods_path = base_path.join("profiles").join(profile_name).join("mods");
    match fs::metadata(&mods_path){
        Ok(_) => return Ok(()),
        Err(_) => {
            Ok(fs::create_dir_all(&mods_path)?)
        }
    }
}
// pub fn read_launcher_profiles(base_path:&PathBuf)->Result<Option<&Map<&String,&Value>>,io::Error>{
//     Ok(Some(json["profiles"].as_object()))
// }
pub fn install_launcher_profile(base_path:&PathBuf,profile_name:&str)->Result<(),Error>{
    let mut launcher_profile = LauncherProfile::from_file(base_path, profile_name);
    // let mut LauncherProfiles = LauncherProfiles::from_file(base_path);
    launcher_profile.save_file(&base_path);
    Ok(())
}
// pub fn create_launcher_profile(base_path:&PathBuf,profile_path:&PathBuf,profile_name:&str)->Result<(),io::Error>{
//     let launcher_profile = LauncherProfile::from_file();
//     Ok(())
// }
pub fn open_profile_location(base_path:&PathBuf,profile_name:&str)->Result<(),io::Error>{
    // println!("{:?}",env::consts::OS);
    let profile_path = base_path.join("profiles").join(profile_name);
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
    use super::*;
    const BASE_PATH_STRING: &str = "test\\.minecraft";
    #[test]
    fn test_insert_profile(){
        let mut launcher_profiles = LauncherProfiles::from_file(&PathBuf::from(BASE_PATH_STRING));
        let launcher_profile = LauncherProfile::from("test_profile");
        launcher_profiles.insert_profile(launcher_profile,&PathBuf::from(BASE_PATH_STRING),"test_profile");
    }

    #[test]
    fn list_mods(){
        let mods_path = PathBuf::from("test").join(".minecraft").join("profiles").join("test_profile");
        let mods = list_profiles_mods(&mods_path).unwrap();
        assert_eq!(mods,[mods_path.join("mods").join("testjar.jar")])
    }
    #[test]
    fn test_create_profile(){
        let base_path = PathBuf::from(BASE_PATH_STRING);
        let profile_name = "new_profile";
        create_profile(&base_path, profile_name).expect("Could not create new Profile!");
        let meta_data = fs::metadata(base_path.join("profiles").join(profile_name)).unwrap();
        assert!(meta_data.is_dir())
    }
    #[test]
    fn test_connect_profile(){
        let mut installer_config = InstallerConfig::new();
        installer_config.sftp_username = Some("headless".parse().unwrap());
        installer_config.sftp_password = Some("pword".parse().unwrap());
        installer_config.sftp_server = Some("bigbrainedgamers.com".parse().unwrap());
        installer_config.sftp_port = Some("2222".parse().unwrap());
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
    fn print(){
        let base_path = PathBuf::from("C:\\Users\\Jeremy\\Documents\\GitHub\\mod-installer\\src-tauri\\test\\.minecraft");
        open_profile_location(&base_path,"test_profile").unwrap()
    }

    #[test]
    fn test_create_config(){
        let installer_config = InstallerConfig::new();
        assert!(installer_config.save_config().is_ok());
    }
    #[test]
    fn test_read_config(){
        let installer_config = InstallerConfig::open();
        assert!(installer_config.default_game_dir.eq(&Some("".parse().unwrap())));
    }
}