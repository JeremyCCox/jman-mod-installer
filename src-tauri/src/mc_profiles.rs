use std::{env, fs, io};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::{Date, DateTime, Utc};


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

pub struct LauncherProfile {
    created :DateTime<Utc>,
    game_dir : PathBuf,
    icon : String,
    last_used :DateTime<Utc>,
    last_version_id : String,
    name : String,
    // profile_type : String, // Should be type but type is reserved in rust!
}
impl LauncherProfile{
    fn new() -> Self{
        Self{
            created: Utc::now(),
            game_dir: Default::default(),
            icon: "".to_string(),
            last_used: Default::default(),
            last_version_id: "".to_string(),
            name: "".to_string(),
        }
    }
}

pub fn create_profile(base_path:&PathBuf,profile_name:&str)-> Result<(),io::Error>{
    let profile_path = PathBuf::from(&base_path).join("profiles").join(profile_name);
    fs::create_dir_all(&profile_path).expect("Couldnt create the profile directory");
    fs::copy(&base_path.join("options.txt"),&profile_path.join("options.txt")).expect("Could not create options copy");
    // let profile_config=  File::create(profile_path.join("profile_config.json")).unwrap();
    // let mut data = String::new();
    // launcher_profiles.read_to_string(& mut data).unwrap();

    // Create Launcher_profile.json to load into .minecraft/launcher_profiles on install
    let local_launcher_profiles = File::open(&base_path.join("launcher_profiles.json")).unwrap();
    let json:serde_json::Value = serde_json::from_reader(&local_launcher_profiles).expect("Could not read JSON from file");
    let launcher_profile =
    for (key,value) in json["profiles"].as_object().unwrap(){
        match value["gameDir"].as_str(){
            None => {}
            Some(x) if PathBuf::from(x).eq(&profile_path) => {
                let obj = value["lastVersionId"].as_str().unwrap();
                println!("{} {}",key,obj)
            }
            x => {}
        }
    }
    println!("{:?}",json);

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
}