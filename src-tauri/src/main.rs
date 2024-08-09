// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{PathBuf};
use serde_json::{json, Value};
use crate::sftp::{RemoteProfile, sftp_download_specific_mods, sftp_install_profile, sftp_list_dir, sftp_read_remote_profiles, sftp_read_specific_remote_profile, sftp_upload_profile, sftp_upload_specific_mods};
use crate::mc_profiles::{copy_local_profile, create_profile, InstallerConfig, open_profile_location};

mod sftp;
mod mc_profiles;

#[tauri::command]
fn read_installer_config()->Result<InstallerConfig,String>{
    match InstallerConfig::open(){
        Ok(config) => { Ok(config) }
        Err(_) => {Err("Could not open installer config!".parse().unwrap())}
    }
}

#[tauri::command]
fn write_installer_config(installer_config: InstallerConfig)-> Result<(), String>{
    println!("{:?}",installer_config);
    match installer_config.save_config(){
        Ok(_)=>Ok(()),
        Err(_)=>Err("could not write to config file!".parse().unwrap())
    }
}



#[tauri::command(async)]
fn attempt_remote_connection_config()->Result<bool,String>{
    let installer_config = match InstallerConfig::open(){
        Ok(installer_config) => { installer_config }
        Err(_) => {
            return Err("Could not open installer config!".parse().unwrap())
        }
    };
    match installer_config.sftp_safe_connect() {
        Ok(_) => {
            Ok(true)
        }
        Err(_) => {
            Err("Could not connect with provided information!".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn attempt_remote_connection_new(installer_config: InstallerConfig)->Result<(),String>{
    match &installer_config.sftp_safe_connect() {
        Ok(_) => {
            let _ = installer_config.save_config();
            Ok(())
        }
        Err(_) => {
            Err("Could not connect with provided information!".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn clear_installer_config()->Result<(),String>{
    match InstallerConfig::clear(){
        Ok(_) => Ok(()),
        Err(_) => {
            Err("Could not delete current config!".parse().unwrap())
        }
    }
}
// #[tauri::command(async)]
// fn delete_local_profile()->Result<(),String>{
//
// }

#[tauri::command(async)]
fn download_sftp_profile(base_path:&str,profile_name:&str)->Result<(),String>{
    println!("{:?}",base_path);
    match sftp_install_profile(&PathBuf::from(base_path),profile_name){
        Ok(_) => Ok(()),
        Err(_) => {
            Err("Could not install profile mods".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn install_missing_mods(base_path:&str,profile_name:&str,missing_mods:Vec<String>)->Result<(),String>{
    match sftp_download_specific_mods(&PathBuf::from(base_path),profile_name,missing_mods) {
        Ok(_) => Ok(()),
        Err(_) => {
            Err("Could not open profile location!".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn read_sftp_dir() -> Result<Value,String> {
    let list_dir = sftp_read_remote_profiles().expect("Could not list directory!");
    Ok(json!(list_dir))
}
#[tauri::command(async)]
fn read_profile_names()->Result<Vec<String>,String>{
    let mut profile_names:Vec<String> = Vec::new();
    let sftp_dir = sftp_list_dir(&PathBuf::from("upload/profiles/")).or_else(|err|{Err("Could not list Profiles")}).unwrap();
    for x in sftp_dir {
        if x.1.is_dir(){
            profile_names.push(x.0.file_name().unwrap().to_os_string().into_string().unwrap())
        }
    }
    Ok(profile_names)
}
#[tauri::command(async)]
fn read_specific_remote_profile(profile_name:&str)->Result<RemoteProfile,String>{
    match sftp_read_specific_remote_profile(profile_name){
        Ok(profile) => {Ok(profile)}
        Err(_) => {Err("Could not read remote profile!".parse().unwrap())}
    }
}
#[tauri::command(async)]
fn upload_sftp_dir(base_path:&str,profile_name:&str)->Result<(),String>{
    match sftp_upload_profile(&PathBuf::from(base_path),profile_name){
        Ok(_) => {
            Ok(())
        }
        Err(error) => {
            Err(error.to_string().into())
        }
    }
}
#[tauri::command(async)]
fn upload_additional_mods(base_path:&str,profile_name:&str,missing_mods:Vec<String>)->Result<(),String>{
    match sftp_upload_specific_mods(&PathBuf::from(base_path),profile_name,missing_mods) {
        Ok(_) => Ok(()),
        Err(_) => {
            Err("Could not upload those mods!".parse().unwrap())
        }
    }
}

#[tauri::command(async)]
fn profile_location(base_path:&str,profile_name:&str)->Result<(),String>{
    let profile_path = PathBuf::from(base_path);
    match open_profile_location(&profile_path,profile_name){
        Ok(_) => {
            Ok(())
        }
        Err(_) => {
            Err("Could not open profile location!".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn create_new_profile(base_path:&str,profile_name:&str)->Result<(),String>{
    match create_profile(&PathBuf::from(base_path),profile_name){
        Ok(_) => {
            Ok(())
        }
        Err(_) => {
            Err("Could not open profile location!".parse().unwrap())
        }
    }
}
#[tauri::command(async)]
fn copy_profile(base_path:&str,profile_name:&str,copy_name:&str)->Result<(),String>{
    match copy_local_profile(base_path,profile_name,copy_name){
        Ok(_) => {
            Ok(())
        }
        Err(_) => {
            Err("Could not open profile location!".parse().unwrap())
        }
    }
}
#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![
          upload_sftp_dir,
          read_sftp_dir,
          greet,
          download_sftp_profile,
          profile_location,
          create_new_profile,
          install_missing_mods,
          upload_additional_mods,
          read_installer_config,
          write_installer_config,
          attempt_remote_connection_config,
          attempt_remote_connection_new,
          read_profile_names,
          read_specific_remote_profile,
      ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
