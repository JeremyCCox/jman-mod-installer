// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::{PathBuf};
use serde_json::{json, Value};
use crate::mc_profiles::{create_profile, open_profile_location};
use crate::sftp::{sftp_download_specific_mods, sftp_install_profile, sftp_read_remote_profiles, sftp_upload_profile, sftp_upload_specific_mods};

mod sftp;
mod mc_profiles;

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
    // let dir_iter = list_dir.iter();
    // let mut dir_readout = Vec::new();
    // for val in dir_iter{
    //     dir_readout.push(&val.0);
    // };
    // Ok(json!(dir_readout))
}
#[tauri::command(async)]
fn upload_sftp_dir(base_path:&str,profile_name:&str)->Result<(),String>{
    // let profile_path = base_path).join("profiles").join(profile_name);
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
#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![upload_sftp_dir,read_sftp_dir,greet,download_sftp_profile,profile_location,create_new_profile,install_missing_mods,upload_additional_mods])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
