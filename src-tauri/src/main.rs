// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ops::Deref;
use serde_json::{json, Value};
use crate::mc_profiles::list_profiles_mods;
use crate::sftp::{sftp_create_profile, sftp_list_dir, sftp_upload_file, sftp_upload_profile_mods};

mod sftp;
mod mc_profiles;

#[tauri::command]
fn read_sftp_dir(path: &str) -> Value {
    println!("{}",path);
    let list_dir = sftp_list_dir(path).unwrap();
    let dir_iter = list_dir.iter();
    let mut dir_readout = Vec::new();
    for val in dir_iter{
        dir_readout.push(&val.0);
    };
    json!(dir_readout)
}
#[tauri::command]
fn upload_sftp_dir(base_path:&str,profile_name:&str){
    let mods = list_profiles_mods([base_path,"profiles",profile_name].join("/").as_str());
    sftp_upload_profile_mods([base_path,"profiles",profile_name].join("/").as_str(),profile_name);
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![greet])
      .invoke_handler(tauri::generate_handler![read_sftp_dir])
      .invoke_handler(tauri::generate_handler![upload_sftp_dir])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
