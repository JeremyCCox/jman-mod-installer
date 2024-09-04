use std::{env, fs};
use std::io::{Write};
use std::path::{PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::installer::{InstallerConfig,InstallerError};
use crate::launcher::{LauncherProfile, LauncherProfiles};

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
    println!("{:?}",profile_path);
    match env::consts::OS{
        x if x.eq("windows")=>{
            Command::new("explorer").arg(profile_path.as_path()).spawn().unwrap();
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
    use crate::addons::AddonType;
    use crate::profiles::local_profile::LocalProfile;
    use crate::profiles::Profile;
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
        let result = &local_profile.install_addons(first_mod,AddonType::Mod);
        println!("{:?}",local_profile);
        assert!(result.is_ok());
        assert_eq!(local_profile.mods.as_ref().unwrap().len(), 1);

        let second_mods = Vec::from(["testjar.jar"]);
        let _ = &local_profile.install_addons(second_mods,AddonType::Mod);
        assert_eq!(local_profile.mods.as_ref().unwrap().len(),2);

    }
    #[test]
    fn list_mods(){
        let mods_path = PathBuf::from("test").join(".minecraft").join("profiles").join("test_profile");
        let mods = list_profiles_mods(&mods_path).unwrap();
        assert_eq!(mods,[mods_path.join("mods").join("testjar.jar")])
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


}