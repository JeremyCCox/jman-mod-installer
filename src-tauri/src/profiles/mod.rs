use crate::addons::{AddonType, ProfileAddon};
use crate::installer::InstallerError;
use crate::profiles::local_profile::LocalProfile;
use crate::profiles::remote_profile::RemoteProfile;

pub mod local_profile;
pub mod remote_profile;

const SFTP_PROFILES_DIR: &str = "/upload/profiles/";

#[derive(Debug)]
#[allow(dead_code)]
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
impl From<RemoteProfile> for LocalProfile{
    fn from(value: RemoteProfile) -> LocalProfile {
        Self{
            name: value.name,
            mods: value.mods,
            version:value.version,
            launcher_profile: value.launcher_profile,
            resource_packs: value.resource_packs,
            config: value.config,
        }
    }
}
pub trait Profile{
    fn new(profile_name:&str)->Self;
    fn create (profile_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn scaffold(&self) ->Result<(),InstallerError>;
    fn open(profile_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn copy (self,copy_name:&str)->Result<Self,InstallerError> where Self: Sized;
    fn delete(self)->Result<(),InstallerError>;
    fn read_addons(&mut self,addon_type: AddonType)->Result<(),InstallerError>;
    fn get_type_addons(&self, addon_type: AddonType)->Result<Vec<ProfileAddon>,InstallerError>;
    fn set_type_addons(&mut self, addons:Vec<ProfileAddon>, addon_type: AddonType) ->Result<(),InstallerError>;
    fn write_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn read_launcher_profile(&mut self)->Result<(),InstallerError>;
    fn rename_profile(&mut self,new_name:&str)->Result<(),InstallerError>;

}

