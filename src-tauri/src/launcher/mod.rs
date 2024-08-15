use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::installer::{InstallerConfig,InstallerError};

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
