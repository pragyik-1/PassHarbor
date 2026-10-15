use tauri::Manager;
use master_key::MasterKey;
use std::sync::{Arc, Mutex};

pub mod error;
pub mod prelude;
mod encryptor;
mod database;
mod master_key;
mod frontend_api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .setup(move|app| {
        app.manage(Mutex::new(MasterKey::new([0u8; 32]))); // Initialize the master key with a zeroed array

        Ok(())
    })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![frontend_api::set_master_key_cmd])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
