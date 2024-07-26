use std::fs::File;
use std::{fs, io};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path,PathBuf};
use ssh2::{Error, FileStat, Session, Sftp};
use crate::mc_profiles::list_profiles_mods;

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
pub fn sftp_list_dir(path:&str) -> Result<Vec<(PathBuf, FileStat)>, Error>{
    let sftp = sftp_connect().unwrap();
    let sftp_home= Path::new(path);
    sftp.readdir(sftp_home)
}
pub fn sftp_save_file(path_string:&String,file_name:&String) {
    let sftp = sftp_connect().unwrap();
    let sftp_home= Path::new(path_string);
    println!("{}", path_string);
    let mut opened_file = sftp.open(sftp_home).expect("File could not be located!");
    let mut write_file = File::create(file_name).expect("Could not create write file!");
    io::copy(&mut opened_file, &mut write_file).expect("Could not save downloaded file!");
}
pub fn sftp_upload_file(local_path: &String, remote_path:&String) {
    let sftp = sftp_connect().unwrap();
    let mut upload_file = fs::File::open(local_path).expect("Could not find File!");
    let mut remote_file = sftp.create(remote_path.as_ref()).expect("Could not create File");
    io::copy(&mut upload_file, &mut remote_file).expect("Could not write file!");
}
pub fn sftp_upload_profile_mods(profile_path:&str, profile_name:&str) {
    sftp_create_profile(profile_name);
    let mods = list_profiles_mods(profile_path);
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
pub fn sftp_create_profile(profile_name: &str){
    let sftp = sftp_connect().unwrap();
    let profile_path = Path::new("/upload/profiles/").join(profile_name);
    sftp.mkdir(&*profile_path, 1000).ok();
    sftp.mkdir(&*profile_path.join("mods"), 1000).ok();
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
        let result = sftp_list_dir("/upload").unwrap();
        let it = result.iter();
        for i in it{
            // let display = i.display();
            println!("{i:?}")
        }
        assert!(true);
    }
    #[test]
    fn upload_file(){
        let remote_path = String::from("/upload/test.file");
        let local_path = String::from("./upload.file");
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
        sftp_create_profile("test_profile");
        let result = sftp_list_dir("/upload/profiles").expect("Dir wasnt found!");
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
        let profile_path = "fake.minecraft/profiles/test_profile/" ;
        let profile_name = "test_profile";
        sftp_upload_profile_mods(profile_path,profile_name);
        let result = sftp_list_dir("/upload/profiles/test_profile/mods").expect("Dir wasnt found!");
        let it = result.iter();
        let mut result_profiles = Vec::new();
        for i in it{
            let pb = i.0.to_str().unwrap();
            result_profiles.push(pb);
            println!("{i:?}")
        }
        assert!(result_profiles.contains(&"/upload/profiles/test_profile/mods\\testjar.jar"));
    }
}
