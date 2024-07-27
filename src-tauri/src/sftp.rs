use std::fs::File;
use std::{fs, io};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path,PathBuf};
use ssh2::{Error, FileStat, Session, Sftp};
use crate::mc_profiles::{create_mods_folder, create_profile, list_profiles_mods};

pub fn sftp_connect() -> Result<Sftp, Error>{
    let tcp = TcpStream::connect("192.168.0.29:2222").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("headless", "pword").unwrap();

    match sess.sftp() {
        Ok(sftp) => Ok(sftp),
        Err(error) => panic!("Sftp connection failed, gave error{:?}",error)
    }
}
pub fn sftp_list_dir(path: &Path) -> Result<Vec<(PathBuf, FileStat)>, Error>{
    let sftp = sftp_connect().expect("Could not connect!");
    sftp.readdir(path)
}
pub fn sftp_save_file(path_string:&String,file_name:&String) {
    let sftp = sftp_connect().unwrap();
    let sftp_home= Path::new(path_string);
    println!("{}", path_string);
    let mut opened_file = sftp.open(sftp_home).expect("File could not be located!");
    let mut write_file = File::create(file_name).expect("Could not create write file!");
    io::copy(&mut opened_file, &mut write_file).expect("Could not save downloaded file!");
}
pub fn sftp_upload_file(local_path: &PathBuf, remote_path:&PathBuf) {
    let sftp = sftp_connect().unwrap();
    let mut upload_file = fs::File::open(local_path).expect("Could not find File!");
    let mut remote_file = sftp.create(remote_path.as_ref()).expect("Could not create File");
    io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
}
pub fn sftp_upload_profile_mods(profile_path:&PathBuf, profile_name:&str) {
    sftp_create_profile(&profile_name).expect("Could not create SFTP profile");
    let mods = list_profiles_mods(profile_path).unwrap();
    let sftp = sftp_connect().unwrap();
    let iter = mods.iter();
    let mods_path = PathBuf::from("/upload/profiles/").join(profile_name).join("mods");
    // println!("{:?}",mods_path)
    for a in iter{
        let mod_path = mods_path.join(a.file_name().unwrap());
        let mut upload_file = fs::File::open(&a).expect("Could not find File!");
        let mut remote_file = sftp.create(mod_path.as_path()).expect("Could not create File");
        io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
    }
}
pub fn sftp_download_mod(sftp:&Sftp){

}
pub fn sftp_download_profile_mods(base_path:&PathBuf, profile_name:&str) ->Result<(),io::Error>{

    create_profile(&base_path,profile_name).expect("Could not create local Profile");
    create_mods_folder(&base_path,profile_name).expect("Error creating mods folder!");
    println!("Profile & Mods folder created");

    let sftp = sftp_connect().unwrap();
    let remote_path = PathBuf::from("/upload/profiles/").join(profile_name).join("mods");
    let local_path = &base_path.join("profiles").join(profile_name).join("mods");
    let mods = sftp_list_dir(&remote_path.as_path()).expect("Could not list Mods Dir");
    let iter = mods.iter();
    for a in iter{
        let file_name = &a.0.file_name().unwrap();
        let mut remote_file = sftp.open(&remote_path.join(&file_name)).expect("Could not create File");
        let mut local_file = fs::File::create(local_path.join(&file_name).as_path()).expect("Could not find File!");
        io::copy(&mut remote_file, &mut local_file).expect("Could not write file!");
    };
    Ok(())
}
pub fn sftp_create_profile(profile_name: &str) -> Result<(), io::Error> {
    let sftp = sftp_connect().unwrap();
    let profile_path = PathBuf::from("upload/profiles").join(profile_name);
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
pub fn sftp_create_mods(path:&PathBuf){

}
#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use super::*;

    #[test]
    fn it_works() {
        let result = sftp_connect();
        assert!(result.is_ok());
    }
    #[test]
    fn list_files() {
        let result = sftp_list_dir(PathBuf::from("/upload").as_path()).unwrap();
        let it = result.iter();
        for i in it{
            // let display = i.display();
            println!("{i:?}")
        }
        assert!(true);
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
    fn create_profile() {
        sftp_create_profile("new_profile").expect("Error creating profile in SFTP");
        let result = sftp_list_dir(PathBuf::from("/upload/profiles").as_path()).expect("Dir wasnt found!");
        let it = result.iter();
        let mut result_profiles = Vec::new();
        for i in it{
            let pb = i.0.to_str().unwrap();
            result_profiles.push(pb);
            println!("{i:?}")
        }
        assert!(result_profiles.contains(&"/upload/profiles\\test_profile"));
    }
    #[test]
    fn upload_profile_mods() {
        let profile_path = PathBuf::from("test/.minecraft/profiles/new_profile/");
        let sftp_profile_path = PathBuf::from("/upload/profiles/new_profile/mods");
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
    fn download_profile_mods(){
        let base_path = PathBuf::from("test/.minecraft");
        let profile_path = PathBuf::from("test/.minecraft/profiles/");
        let profile_name = "new_profile";
        sftp_download_profile_mods(&base_path,profile_name);
        let result = list_profiles_mods(&profile_path.join(profile_name)).expect("Dir wasn't found!");
        let it = result.iter();
        let mut result_profiles:Vec<&PathBuf> = Vec::new();
        for i in it{
            let pb = i;
            result_profiles.push(pb);
        }
        assert!(result_profiles.contains(&&profile_path.join(profile_name).join("mods/testjar.jar")));
        fs::remove_dir_all(profile_path.join(profile_name)).unwrap();
    }
}
