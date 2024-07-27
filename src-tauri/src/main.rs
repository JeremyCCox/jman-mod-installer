// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::mc_profiles::{list_profiles_mods, open_profile_location};
use crate::sftp::{sftp_download_profile_mods, sftp_list_dir, sftp_upload_profile_mods};

mod sftp;
mod mc_profiles;

#[tauri::command]
fn download_sftp_profile(base_path:&str,profile_name:&str){
    println!("{:?}",base_path);
    sftp_download_profile_mods(&PathBuf::from(base_path),profile_name).expect("Could not download profile mods");
}
#[tauri::command]
fn read_sftp_dir(path:&str) -> Value {
    let list_dir = sftp_list_dir(PathBuf::from(path).as_path()).expect("Could not list directory!");
    let dir_iter = list_dir.iter();
    let mut dir_readout = Vec::new();
    for val in dir_iter{
        dir_readout.push(&val.0);
    };
    json!(dir_readout)
}
#[tauri::command]
fn upload_sftp_dir(base_path:&str,profile_name:&str){
    let profile_path = PathBuf::from(base_path).join("profiles").join(profile_name);
    let mods = list_profiles_mods(&profile_path);
    sftp_upload_profile_mods(&profile_path,profile_name);
}
#[tauri::command]
fn profile_location(base_path:&str,profile_name:&str){
    let profile_path = PathBuf::from(base_path);
    open_profile_location(&profile_path,profile_name).expect("Could not open File Explorer!");
}
#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![upload_sftp_dir,read_sftp_dir,greet,download_sftp_profile,profile_location])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
