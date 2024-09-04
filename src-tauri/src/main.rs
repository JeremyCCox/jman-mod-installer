// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::{PathBuf};
use serde_json::{json, Value};


use crate::installer::{InstallerConfig, InstallerError};
use crate::launcher::LauncherProfiles;
use crate::mc_profiles::open_profile_location;
use crate::mods::{AddonManager, Mod};
use crate::profiles::local_profile::LocalProfile;
use crate::profiles::{local_profile, Profile};
use crate::sftp::{ sftp_list_dir, sftp_read_remote_profiles};
use crate::profiles::remote_profile::RemoteProfile;
use crate::resource_packs::{PackManager, ResourcePack};

mod sftp;
mod mc_profiles;
mod profiles;

mod launcher;
mod installer;
mod resource_packs;
mod mods;

#[tauri::command]
fn read_installer_config()->Result<InstallerConfig,String>{
   Ok(InstallerConfig::open()?)
}

#[tauri::command]
fn write_installer_config(installer_config: InstallerConfig)-> Result<(), String>{
    println!("{:?}",installer_config);
    Ok(installer_config.save_config()?)
}

#[tauri::command(async)]
fn attempt_remote_connection_config()->Result<bool,String>{
    let installer_config =  InstallerConfig::open()?;
    Ok(installer_config.sftp_safe_connect().is_ok())
}
#[tauri::command(async)]
fn attempt_remote_connection_new(installer_config: InstallerConfig)->Result<bool,String>{
    match &installer_config.sftp_safe_connect(){
        Ok(_) => {
            let _ = installer_config.save_config();
            Ok(true)
        }
        Err(err) => {
            // Ok(false)
            Err(err.to_string())
        }
    }
}
#[tauri::command(async)]
fn clear_installer_config()->Result<(),String>{
    Ok(InstallerConfig::clear()?)
}

#[tauri::command(async)]
fn download_sftp_profile(profile_name:&str)->Result<(),String>{
    let remote_profile =RemoteProfile::open(profile_name)?;
    dbg!(remote_profile.install_profile()?);
    Ok(())
}
#[tauri::command(async)]
fn delete_local_profile(profile_name:&str)->Result<(),String>{
    let local_profile = LocalProfile::open(profile_name)?;
    Ok(local_profile.delete()?)
}
#[tauri::command(async)]
fn install_missing_mods(profile_name:&str,mods_list:Vec<&str>)->Result<(),String>{
    Ok(LocalProfile::open(profile_name).unwrap().install_mods(mods_list)?)
}
#[tauri::command]
fn install_new_mods(profile: &str,mod_list:Vec<Mod>)->Result<(),InstallerError>{
    let mut local_profile = LocalProfile::open(profile)?;
    local_profile.install_new_mods(mod_list)?;
    Ok(())
}
#[tauri::command(async)]
fn install_specified_mods(profile_name:&str,mods_list:Vec<&str>)->Result<(),String>{
    let mut local_profile = LocalProfile::open(profile_name).unwrap();
    Ok(local_profile.install_mods(mods_list)?)
}
#[tauri::command(async)]
fn install_resource_pack(profile_name:&str,pack_name:&str)->Result<(),String>{
    let mut local_profile = LocalProfile::open(profile_name)?;
    Ok(local_profile.add_resource_pack(pack_name)?)
}
#[tauri::command(async)]
fn remove_local_resource_pack(profile_name:&str,pack_name:&str)->Result<(),String>{
    let mut local_profile = LocalProfile::open(profile_name)?;
    Ok(local_profile.delete_resource_pack(pack_name)?)
}
#[tauri::command(async)]
fn read_sftp_dir() -> Result<Value,String> {
    let list_dir = sftp_read_remote_profiles().expect("Could not list directory!");
    Ok(json!(list_dir))
}
#[tauri::command(async)]
fn list_local_profiles()->Result<Vec<String>,String>{
    let readout = fs::read_dir(InstallerConfig::open()?.default_game_dir.unwrap().join("profiles")).unwrap();
    let mut profiles_list= Vec::new();
    let launcher_profiles = LauncherProfiles::open();
    for x in readout {
        let name = x.unwrap().file_name().into_string().unwrap();
        if launcher_profiles.profiles.contains_key(&name){
            profiles_list.push(name)
        }
    }
    println!("{:?}",profiles_list);
    Ok(profiles_list)
}
#[tauri::command(async)]
fn list_remote_profiles()->Result<Vec<String>,String>{
    let mut profile_names:Vec<String> = Vec::new();
    let sftp_dir = sftp_list_dir(&PathBuf::from("upload/profiles/")).unwrap();
    for x in sftp_dir {
        if x.1.is_dir(){
            profile_names.push(x.0.file_name().unwrap().to_os_string().into_string().unwrap())
        }
    }
    Ok(profile_names)
}
#[tauri::command(async)]
fn read_specific_remote_profile(profile_name:&str)->Result<RemoteProfile,String>{
    println!("Heah");
    Ok(RemoteProfile::open(profile_name)?)
}
#[tauri::command(async)]
fn read_specific_local_profile(profile_name:&str)->Result<LocalProfile,String> {
    Ok(LocalProfile::open(profile_name)?)
}
#[tauri::command(async)]
fn read_remote_resource_packs()->Result<Vec<ResourcePack>,InstallerError>{
    Ok(AddonManager::read_remote_packs()?)
}
#[tauri::command(async)]
fn read_remote_mods()->Result<Vec<Mod>,InstallerError>{
    Ok(AddonManager::read_remote_mods()?)
}
#[tauri::command(async)]
fn verify_profile_files(profile_name:&str)->Result<LocalProfile,InstallerError>{
    let mut lp =  LocalProfile::open(profile_name)?;
    lp.verify_profile_files()?;
    Ok(lp)
}
#[tauri::command(async)]
fn upload_local_profile(profile_name:&str)->Result<(),String>{
    let local_profile = LocalProfile::open(profile_name).unwrap();
    local_profile.upload_profile()?;
    Ok(())
}
#[tauri::command(async)]
fn upload_additional_mods(profile_name:&str,mods_list:Vec<Mod>)->Result<(),String>{
    let local_profile = LocalProfile::open(profile_name)?;
    dbg!(&mods_list);
    Ok(local_profile.upload_mods(mods_list)?)
}

#[tauri::command(async)]
fn profile_location(profile_name:&str)->Result<(),String>{
    Ok(open_profile_location(profile_name)?)
}
#[tauri::command(async)]
fn create_new_profile(profile_name:&str)->Result<LocalProfile,String>{
    Ok(LocalProfile::create(profile_name)?)
}
#[tauri::command(async)]
fn copy_local_profile(profile_name:&str,copy_name:&str)->Result<LocalProfile,String>{
    Ok(LocalProfile::open(profile_name)?.copy(copy_name)?)
}
#[tauri::command(async)]
fn copy_remote_profile(profile_name:&str,copy_name:&str)->Result<RemoteProfile,String>{
    Ok(RemoteProfile::open(profile_name)?.copy(copy_name)?)
}
#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![
          upload_local_profile,
          read_sftp_dir,
          greet,
          download_sftp_profile,
          profile_location,
          create_new_profile,
          clear_installer_config,
          install_missing_mods,
          install_new_mods,
          install_specified_mods,
          install_resource_pack,
          remove_local_resource_pack,
          upload_additional_mods,
          read_installer_config,
          write_installer_config,
          attempt_remote_connection_config,
          attempt_remote_connection_new,
          list_local_profiles,
          list_remote_profiles,
          read_specific_remote_profile,
          read_specific_local_profile,
          read_remote_resource_packs,
          read_remote_mods,
          verify_profile_files,
          delete_local_profile,
          copy_local_profile,
          copy_remote_profile,
      ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
