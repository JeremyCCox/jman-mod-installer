use std::fs::File;
use std::{fs, io};
use std::io::{Write};
use std::path::{Path,PathBuf};
use serde::{Deserialize, Serialize};
use ssh2::{Error, FileStat};
use crate::mc_profiles::{create_mods_folder, create_profile, InstallerConfig, LauncherProfile, LauncherProfiles, list_profiles_mods};


const SFTP_PROFILES_DIR: &str = "/upload/profiles/";

#[derive(Serialize,Deserialize,Debug)]
pub struct RemoteProfileInfo{
    pub name:String,
    pub mods:Option<Vec<String>>,
    pub launcher_profile:Option<LauncherProfile>,
}
impl RemoteProfileInfo{

    pub fn new(name: String) ->Self{
        Self{
            name:name,
            mods: None,
            launcher_profile: None,
        }
    }
    // pub fn from_sftp(name:String)->Self{
    //     Self{
    //         name,
    //         mods: None,
    //         launcher_profile: None,
    //     }
    // }
}

pub fn sftp_list_dir(path: &Path) -> Result<Vec<(PathBuf, FileStat)>, Error>{
    let sftp =  InstallerConfig::open().unwrap().sftp_safe_connect().expect("Could not connect!");
    sftp.readdir(path)
}
pub fn sftp_list_mods(profile_name:&str)->Result<Vec<String>,ssh2::Error>{
    let sftp =  InstallerConfig::open().unwrap().sftp_safe_connect().expect("Could not establish SFTP connection");

    let mut mods_list = Vec::new();
    match sftp.readdir(PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods").as_path()) {
        Ok(dir_readout) => {
            for i in dir_readout.iter(){
                let file_name = i.0.file_name().unwrap();
                mods_list.push(file_name.to_str().unwrap().to_string())
            }

            Ok(mods_list)
        }
        Err(err) => {
            Err(err)
        }
    }
}
pub fn sftp_read_remote_profiles()->Result<Vec<RemoteProfileInfo>,ssh2::Error>{
    // let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
    let mut remote_profiles:Vec<RemoteProfileInfo> = Vec::new();
    match sftp_list_dir(PathBuf::from(SFTP_PROFILES_DIR).as_path()){
        Ok(readout) => {
            for i in readout.iter(){
                let mut remote_profile = RemoteProfileInfo::new(String::from(i.0.file_name().unwrap().to_str().unwrap()));
                remote_profile.mods = Some(sftp_list_mods(remote_profile.name.as_str()).unwrap());
                remote_profile.launcher_profile = Some(sftp_read_launcher_profile(remote_profile.name.as_str()).unwrap());
                remote_profiles.push(remote_profile);
            }
        }
        Err(_) => {}
    }
    Ok(remote_profiles)
}
pub fn sftp_save_file(path_string:&String,file_name:&String) {
    let installer_config = InstallerConfig::open().unwrap();
    let sftp =installer_config.sftp_safe_connect().unwrap();
    let sftp_home= Path::new(path_string);
    println!("{}", path_string);
    let mut opened_file = sftp.open(sftp_home).expect("File could not be located!");
    let mut write_file = File::create(file_name).expect("Could not create write file!");
    io::copy(&mut opened_file, &mut write_file).expect("Could not save downloaded file!");
}
pub fn sftp_upload_file(local_path: &PathBuf, remote_path:&PathBuf) {
    let installer_config = InstallerConfig::open().unwrap();
    let sftp =installer_config.sftp_safe_connect().unwrap();
    let mut upload_file = fs::File::open(local_path).expect("Could not find File!");
    let mut remote_file = sftp.create(remote_path.as_ref()).expect("Could not create File");
    io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
}
pub fn sftp_upload_profile(base_path:&PathBuf, profile_name:&str) ->Result<(),io::Error>{
    sftp_create_profile_dirs(&profile_name)?;
    sftp_create_launcher_profile(&base_path,profile_name)?;
    sftp_upload_mods(&base_path,profile_name)?;
    Ok(())
}
pub fn sftp_upload_mods(base_path:&PathBuf, profile_name:&str)->Result<(),io::Error>{
    let mods = list_profiles_mods(&base_path.join("profiles").join(profile_name)).unwrap();
    let installer_config = InstallerConfig::open()?;
    let sftp =installer_config.sftp_safe_connect().unwrap();
    let iter = mods.iter();
    let mods_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods");
    // println!("{:?}",mods_path)
    for a in iter{
        let mod_path = mods_path.join(a.file_name().unwrap());
        let mut upload_file = fs::File::open(&a).expect("Could not find File!");
        let mut remote_file = sftp.create(mod_path.as_path()).expect("Could not create File");
        io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
    };
    Ok(())
}
pub fn sftp_upload_specific_mods(base_path:&PathBuf, profile_name:&str,missing_list:Vec<String>)->Result<(),io::Error>{
    let mods = list_profiles_mods(&base_path.join("profiles").join(profile_name)).unwrap();
    let installer_config = InstallerConfig::open()?;
    let sftp =installer_config.sftp_safe_connect().unwrap();
    let iter = mods.iter();
    let mods_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods");
    // println!("{:?}",mods_path)
    for a in iter{
        match missing_list.contains(&a.file_name().unwrap().to_str().unwrap().to_string()){
            true => {
                let mod_path = mods_path.join(a.file_name().unwrap());
                let mut upload_file = fs::File::open(&a).expect("Could not find File!");
                let mut remote_file = sftp.create(mod_path.as_path()).expect("Could not create File");
                io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
            }
            false => {
            }
        }
    };
    Ok(())
}
pub fn sftp_upload_profile_mods(profile_path:&PathBuf, profile_name:&str) {
    sftp_create_profile_dirs(&profile_name).expect("Could not create SFTP profile");
    let mods = list_profiles_mods(profile_path).unwrap();
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
    let iter = mods.iter();
    let mods_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods");
    // println!("{:?}",mods_path)
    for a in iter{
        let mod_path = mods_path.join(a.file_name().unwrap());
        let mut upload_file = fs::File::open(&a).expect("Could not find File!");
        let mut remote_file = sftp.create(mod_path.as_path()).expect("Could not create File");
        io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
    }
}
pub fn sftp_create_launcher_profile(base_path:&PathBuf,profile_name:&str)->Result<(),io::Error>{
    let installer_config = InstallerConfig::open()?;
    let sftp =installer_config.sftp_safe_connect().unwrap();

    let launcher_profile:LauncherProfile = LauncherProfile::from_file(base_path,profile_name);
    let remote_launcher_profile = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("launcher_profile.json");
    let launcher_json = serde_json::to_string(&launcher_profile)?;
    let mut launcher_file = sftp.create(remote_launcher_profile.as_path())?;
    launcher_file.write(launcher_json.as_ref()).expect("Could not write launcher_profile.json to remote profile");
    Ok(())
}
// pub fn sftp_download_mod(sftp:&Sftp){
//
// }
pub fn sftp_install_launcher_profile(base_path:&PathBuf,profile_name:&str)->Result<(),io::Error>{
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().expect("Could not establish SFTP connection!");
    let remote_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("launcher_profile.json");
    // let mut profiles_map: &Map<String, Value> = launcher_profiles["profiles"].as_object().unwrap();
    let mut launcher_profiles: LauncherProfiles = LauncherProfiles::from_file(base_path);
    match sftp.lstat(&remote_path.as_path()){
        Ok(_) => {
            let remote_file = sftp.open(&remote_path.as_path())?;
            let launcher_profile:LauncherProfile = serde_json::from_reader(remote_file)?;
            launcher_profiles.insert_profile(launcher_profile,base_path,profile_name);
            launcher_profiles.save(base_path);
            Ok(())
        },
        Err(_) => {
            fs::rename(base_path.join("launcher_profiles.json"),base_path.join("launcher_profiles.json"))?;
            panic!("Could not find remote launcher profile!")
        }
    }
    // Ok(())
}
pub fn sftp_download_profile_mods(base_path:&PathBuf, profile_name:&str) ->Result<(),io::Error>{

    create_profile(&base_path,profile_name).expect("Could not create local Profile");
    create_mods_folder(&base_path,profile_name).expect("Error creating mods folder!");
    let _ = sftp_download_mods(&base_path,profile_name);
    Ok(())
}
pub fn sftp_download_mods(base_path:&PathBuf, profile_name:&str) ->Result<(),io::Error>{
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect()?;
    let remote_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods");
    let local_path = &base_path.join("profiles").join(profile_name).join("mods");
    let mods = sftp_list_dir(&remote_path.as_path()).expect("Could not list Mods Dir");
    let iter = mods.iter();
    for a in iter{
        let file_name = &a.0.file_name().expect("Failed to read file names!");
        let mut remote_file = sftp.open(&remote_path.join(&file_name)).expect("Could not create File");
        let mut local_file = fs::File::create(local_path.join(&file_name).as_path()).expect("Could not find File!");
        io::copy(&mut remote_file, &mut local_file).expect("Could not write file!");
    };
    Ok(())
}
pub fn sftp_download_specific_mods(base_path:&PathBuf, profile_name:&str,mods_list:Vec<String>) ->Result<(),ssh2::Error>{
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect()?;
    let remote_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("mods");
    let local_path = &base_path.join("profiles").join(profile_name).join("mods");
    match sftp_list_dir(&remote_path.as_path()){
        Ok(readout) => {
            for x in readout {
                let file_name = String::from(x.0.file_name().unwrap().to_str().unwrap());
                match mods_list.contains(&file_name){
                    true => {
                        let mut remote_file = sftp.open(&remote_path.join(&file_name)).expect("Could not create File");
                        let mut local_file = fs::File::create(local_path.join(&file_name).as_path()).expect("Could not find File!");
                        io::copy(&mut remote_file, &mut local_file).expect("Could not write file!");
                    }
                    false => {}
                }
            };
            Ok(())
        }
        Err(err) => {Err(err)}
    }
    // Ok(())
}

pub fn sftp_read_launcher_profile(profile_name:&str)->Result<LauncherProfile,ssh2::Error>{
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
    match sftp.open(PathBuf::from(SFTP_PROFILES_DIR).join(profile_name).join("launcher_profile.json").as_path()) {
        Ok(file) => {
            Ok(serde_json::from_reader(file).expect("Could not read JSON from file"))
        }
        Err(err) => {
            Err(err)
        }
    }
}
pub fn sftp_install_profile(base_path:&PathBuf, profile_name:&str)->Result<(),io::Error>{
    create_profile(&base_path,profile_name).expect("Could not create local Profile");
    create_mods_folder(&base_path,profile_name).expect("Error creating mods folder!");
    sftp_download_mods(&base_path,profile_name)?;
    sftp_install_launcher_profile(&base_path,profile_name)?;
    Ok(())
}
pub fn sftp_create_profile_dirs(profile_name: &str) -> Result<(), io::Error> {
    let sftp = InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();
    let profile_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name);
    match sftp.lstat(&profile_path.as_path()){
        Ok(_) => {
            match sftp.lstat(&profile_path.join("mods").as_path()){
                Ok(_) => {Ok(())}
                Err(_) => {
                    Ok(sftp.mkdir(&profile_path.join("mods").as_path(), 1000)?)
                }
            }
        },
        Err(_) => {
                match sftp.mkdir(&profile_path.as_path(), 1000) {
                    Ok(_) => Ok(sftp.mkdir(&profile_path.join("mods").as_path(), 1000)?),
                    Err(_) => panic!("Could not find or create Profile!"),
                }
            }
    }
}
// pub fn sftp_create_mods(path:&PathBuf){
//
// }
#[cfg(test)]
mod tests {
    use serial_test::serial;
    use super::*;

    const LOCAL_BASE_PATH_STRING: &str = "test\\.minecraft";
    // pub fn setup_test_profile()->Result<(),io::Error>{
    //     let installer_config:InstallerConfig = InstallerConfig::test_new();
    //     installer_config.save_config()
    // }
    #[test]
    fn it_works() {
        
        let test_profile = InstallerConfig::open().unwrap();
        assert!(test_profile.sftp_safe_connect().is_ok());
    }
    // #[test]
    // fn list_files() {
    //     let result = sftp_list_dir(PathBuf::from("/upload").as_path()).unwrap();
    //     let it = result.iter();
    //     for i in it{
    //         // let display = i.display();
    //         println!("{i:?}")
    //     }
    //     assert!(true);
    // }
    #[test]
    fn list_mods(){
        
        let result = sftp_list_mods("new_profile").unwrap();
        println!("{:?}",result);
        assert!(result.contains(&String::from("testjar.jar")))

    }
    #[test]
    fn read_launcher_profile(){

    }
    #[test]
    fn upload_file(){
        
        let remote_path = PathBuf::from("/upload/test.file");
        let local_path = PathBuf::from("test/upload.file");
        sftp_upload_file(&local_path, &remote_path);
        assert!(true);
    }
    #[test]
    fn save_file() {
        
        let file_path = String::from("/upload/test.file");
        let file_name = String::from("save.file");
        sftp_save_file(&file_path, &file_name);
        assert!(true);
    }
    #[test]
    fn test_create_profile_dirs() {
        
        sftp_create_profile_dirs("new_profile").expect("Error creating profile in SFTP");
        let result = sftp_list_dir(PathBuf::from(SFTP_PROFILES_DIR).as_path()).expect("Dir wasnt found!");
        let it = result.iter();
        let mut result_profiles = Vec::new();
        for i in it{
            let pb = i.0.to_str().unwrap();
            result_profiles.push(pb);
            println!("{i:?}")
        }
        assert!(result_profiles.contains(&"/upload/profiles/new_profile"));
    }
    #[test]
    fn upload_profile_mods() {
        
        let profile_path = PathBuf::from("test/.minecraft/profiles/new_profile/");
        let sftp_profile_path = PathBuf::from(SFTP_PROFILES_DIR).join("new_profile/mods");
        let profile_name = "new_profile";
        sftp_upload_profile_mods(&profile_path,profile_name);
        let result = sftp_list_dir(sftp_profile_path.as_path()).expect("Dir wasn't found!");
        let it = result.iter();
        let mut result_profiles = Vec::new();
        for i in it{
            let pb = &i.0;
            result_profiles.push(pb);
            println!("{i:?}")
        }
        assert!(result_profiles.contains(&&sftp_profile_path.join("testjar.jar")));
    }
    #[test]
    fn test_upload_specific_mods() {
        
        let base_path = PathBuf::from("test/.minecraft");
        let sftp_profile_path = PathBuf::from(SFTP_PROFILES_DIR).join("new_profile/mods");
        let profile_name = "new_profile";

        let mut missing_mods:Vec<String> = Vec::new();
        missing_mods.push(String::from("testjar.jar"));
        sftp_upload_specific_mods(&base_path, profile_name, missing_mods).expect("Could not upload missing mods");
        let result = sftp_list_dir(sftp_profile_path.as_path()).expect("Dir wasn't found!");
        let it = result.iter();
        let mut result_profiles = Vec::new();
        for i in it{
            let pb = &i.0;
            result_profiles.push(pb);
            println!("{i:?}")
        }
        assert!(result_profiles.contains(&&sftp_profile_path.join("testjar.jar")));
    }
    #[test]
    #[serial]
    fn test_upload_launcher_profile(){
        
        let base_path = PathBuf::from(LOCAL_BASE_PATH_STRING);
        let profile_name = "new_profile";
        // let profile_path = base_path.join("profiles").join(profile_name);
        sftp_create_launcher_profile(&base_path, profile_name).expect("Could not create new Profile!");
        let remote_path = PathBuf::from(SFTP_PROFILES_DIR).join(profile_name);
        let mut remote_files:Vec<&PathBuf> = Vec::new();
        let remote_readout = &sftp_list_dir(remote_path.as_path()).unwrap();
        for i in remote_readout{
            let pb = &i.0;
            remote_files.push(pb);
        }
        assert!(remote_files.contains(&&remote_path.join("launcher_profile.json")));
    }
    #[test]
    #[serial]
    fn test_read_launcher_profile(){
        let launcher_profile = sftp_read_launcher_profile("new_profile").unwrap();
        println!("{:?}",launcher_profile);
        assert!(launcher_profile.name.eq(&Some(String::from("new_profile"))));
    }
    #[test]
    fn test_install_launcher_profile(){
        
        let base_path = PathBuf::from(LOCAL_BASE_PATH_STRING);
        let profile_name = "new_profile";
        assert!(sftp_install_launcher_profile(&base_path, profile_name).is_ok());
        // Test that the launcher_profiles was updated correctly
        let file = File::open(base_path.join("launcher_profiles.json")).unwrap();
        let launcher_profiles: LauncherProfiles = serde_json::from_reader(&file).unwrap();
        let mut profile_names: Vec<&String> = Vec::new();
        for p in launcher_profiles.profiles.iter(){
            profile_names.push(&p.0);
        }
        assert!(profile_names.contains(&&profile_name.to_string()));

    }
    #[test]
    #[serial]
    fn test_download_profile_mods(){
        
        let base_path = PathBuf::from("test/.minecraft");
        let profile_path = PathBuf::from("test/.minecraft/profiles/");
        let profile_name = "new_profile";
        let _ = sftp_download_profile_mods(&base_path,profile_name);
        let result = list_profiles_mods(&profile_path.join(profile_name)).expect("Dir wasn't found!");
        let it = result.iter();
        let mut result_profiles:Vec<&PathBuf> = Vec::new();
        for i in it{
            let pb = i;
            result_profiles.push(pb);
        }
        assert!(result_profiles.contains(&&profile_path.join(profile_name).join("mods/testjar.jar")));
        // fs::remove_dir_all(profile_path.join(profile_name)).unwrap();
    }
    #[test]
    fn test_download_specific_mods(){
        
        let base_path = PathBuf::from("test/.minecraft");
        let profile_name = "new_profile";
        let mut mods_list = Vec::new();
        mods_list.push(String::from("testjar.jar"));

        let mods_path = base_path.join("profiles").join(profile_name).join("mods");

        fs::remove_file(mods_path.join("testjar.jar")).ok();
        assert!(!fs::metadata(mods_path.join("testjar.jar")).is_ok());
        sftp_download_specific_mods(&base_path,profile_name,mods_list).unwrap();
        assert!(fs::metadata(mods_path.join("testjar.jar")).unwrap().is_file());


    }
    #[test]
    #[serial]
    fn test_download_profile(){
        
        let base_path = PathBuf::from("test/.minecraft");
        let profile_name = "new_profile";

        // profile path
        let profile_path = base_path.join("profiles");

        // Test that the function ran without errors
        assert!(sftp_install_profile(&base_path,profile_name).is_ok());

        // Test that the directory was created properly
        assert!(fs::metadata(profile_path.join("new_profile").as_path()).unwrap().is_dir());

        // Test that the launcher_profiles was updated correctly
        let file = File::open(base_path.join("launcher_profiles.json")).unwrap();
        let launcher_profiles: LauncherProfiles = serde_json::from_reader(&file).unwrap();
        let mut profile_names: Vec<&String> = Vec::new();
        for p in launcher_profiles.profiles.iter(){
            profile_names.push(&p.0);
        }
        assert!(profile_names.contains(&&profile_name.to_string()));

        // Test that the mods folder exists and contains testjar.jar
        assert!(fs::metadata(profile_path.join("new_profile").join("mods").join("testjar.jar").as_path()).unwrap().is_file())

    }
    #[test]
    fn test_upload_profile(){
        
        let base_path = PathBuf::from("test/.minecraft");
        let profile_name = "new_profile";

        // SFTP client for tests
        let sftp =  InstallerConfig::open().unwrap().sftp_safe_connect().unwrap();

        // Test that the function ran without errors
        assert!(sftp_upload_profile(&base_path,profile_name).is_ok());

        // Test that the directory was created properly
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").as_path()).unwrap().is_dir());

        // Test that the launcher_profile was created
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").join("launcher_profile.json").as_path()).unwrap().is_file());

        // Test that the mods folder exists and contains testjar.jar
        assert!(sftp.lstat(PathBuf::from(SFTP_PROFILES_DIR).join("new_profile").join("mods").join("testjar.jar").as_path()).unwrap().is_file())

    }
    #[test]
    fn test_read_remote_profiles(){
        
        let remote_profiles = sftp_read_remote_profiles().unwrap();
        let mut names = Vec::new();
        for x in remote_profiles {
            names.push(x.name);
        }
        assert!(names.contains(&String::from("new_profile")))
    }
}
