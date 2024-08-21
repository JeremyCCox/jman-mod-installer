use std::fs::File;
use std::{fs, io};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};

#[derive(Debug,thiserror::Error)]
pub enum InstallerError{
    #[error(transparent)]
    Ssh2(#[from] ssh2::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::error::Error)
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
        sess.userauth_password(&self.sftp_username.clone().unwrap(), &self.sftp_password.clone().unwrap())?;
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
