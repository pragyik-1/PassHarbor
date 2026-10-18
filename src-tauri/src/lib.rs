use std::sync::Mutex;
use tauri::Manager;

mod database;
mod encryptor;
pub mod error;
mod frontend_api;
mod master_key;
pub mod prelude;
mod utils;

struct AppState {
    master_key: Mutex<master_key::MasterKey>,
    enc_key: Mutex<Option<encryptor::EncKey>>,
    db_key: Mutex<Option<database::DbKey>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            master_key: Mutex::new(master_key::MasterKey::new()),
            db_key: Mutex::new(None),
            enc_key: Mutex::new(None),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let app_state = AppState::default();
            app.manage(app_state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            frontend_api::set_master_key_cmd,
            frontend_api::clear_master_key_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
