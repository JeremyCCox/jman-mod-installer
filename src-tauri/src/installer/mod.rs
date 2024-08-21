use std::fs::File;
use std::{fs, io};
use std::error::Error;
use std::fmt::{Display, Formatter, write};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};
use thiserror::Error;

#[derive(Debug,Error)]
struct ConfigError{
    code:usize,
    message:String,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Error code {} message {}",self.code,self.message)
    }
}

impl ConfigError{
    fn no_address()->Self{
        Self{
            code:1,
            message: "no sftp_server found in config!".to_string(),
        }
    }
    fn no_port()->Self{
        Self{
            code:2,
            message: "no sftp_port found in config!".to_string(),
        }
    }fn no_username()->Self{
        Self{
            code:3,
            message: "no sftp_username found in config!".to_string(),
        }
    }
    fn no_password()->Self{
        Self{
            code:4,
            message: "no sftp_password found in config!".to_string(),
        }
    }

}
#[derive(Debug,thiserror::Error)]
pub enum InstallerError{
    #[error(transparent)]
    Ssh2(#[from] ssh2::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::error::Error),
    #[error(transparent)]
    Config(#[from] ConfigError)
}

impl serde::Serialize for InstallerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
impl From<InstallerError> for String{
    fn from(value: InstallerError) -> Self {
        value.to_string()
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
            default_game_dir: Some(PathBuf::from("C:/Users/Jeremy/AppData/Roaming/.minecraft")),
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
                Err(InstallerError::Io(err))
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
                Err(InstallerError::Io(err))
            }
        }
    }
    pub fn sftp_connect(&self)->Result<Sftp,InstallerError>{
        let address = match &self.sftp_server.clone(){
            Some(address)=>address,
            None=>{
                let error = InstallerError::Config(ConfigError::no_address());
                return Err(error)
            }
        };
        let address = format!("{}:{}",&self.sftp_server.clone().expect("Could not read sftp_server"),&self.sftp_port.clone().expect("Could not read sftp_port"));
        let resolved_addresses:Vec<_> = address.to_socket_addrs().expect("Unable to resolve domain")
            .collect();
        let tcp = TcpStream::connect_timeout(resolved_addresses.get(0).unwrap(),Duration::new(25,0))?;

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
                        return Err(InstallerError::Ssh2(error))
                    }
                }
            }
        }
        sess.userauth_password(&self.sftp_username.clone().unwrap(), &self.sftp_password.clone().unwrap())?;
        match sess.sftp() {
            Ok(sftp) => Ok(sftp),
            Err(error) => Err(InstallerError::Ssh2(error))
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
                    println!("{:?}",err);
                    match &err {
                        InstallerError::Ssh2(ssh2) => {
                            if ssh2.code().to_string().eq("Session(-18)"){
                             break Err(err)
                            }
                        }
                        _ => {}
                    }
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
#[cfg(test)]
mod tests{
    use std::fs;
    use std::path::PathBuf;
    use serial_test::serial;
    use crate::installer::{InstallerConfig, InstallerError};
    use crate::mc_profiles::create_mods_folder;
    const BASE_PATH_STRING: &str = "C:/Users/Jeremy/AppData/Roaming/.minecraft";
    const SFTP_PROFILES_DIR: &str = "/upload/profiles/";
    pub fn setup_test_profile()->Result<(),InstallerError>{
        let installer_config:InstallerConfig = InstallerConfig::test_new();
        installer_config.test_save()
    }
    #[test]
    fn test_connect_profile(){
        let installer_config = InstallerConfig::test_new();
        let sftp_result = installer_config.sftp_connect();
        assert!(sftp_result.is_ok());
    }
    #[test]
    fn test_mods_folder(){
        let base_path = PathBuf::from(BASE_PATH_STRING);
        assert!(create_mods_folder(&base_path,"new_mods_folder").is_ok());
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
        assert!(installer_config.default_game_dir.eq(&Some("C:/Users/Jeremy/AppData/Roaming/.minecraft".parse().unwrap())));
    }
}