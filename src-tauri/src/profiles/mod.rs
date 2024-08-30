use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::installer::InstallerError;
use crate::launcher::LauncherProfiles;
use crate::profiles::local_profile::LocalProfile;
use crate::profiles::remote_profile::RemoteProfile;

pub mod local_profile;
pub mod remote_profile;

const SFTP_PROFILES_DIR: &str = "/upload/profiles/";


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
    fn read_resource_packs(&mut self)->Result<(),InstallerError>;
    fn write_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn read_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn rename_profile(&mut self,new_name:&str)->Result<(),InstallerError>;

}

pub trait ProfileAddon{
    fn new(name:&str)->Self;
    fn open_remote(name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn open_local(name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn upload(&self, source:&PathBuf) ->Result<(),InstallerError>;
    fn download(&self, location:&PathBuf) ->Result<(),InstallerError>;
}