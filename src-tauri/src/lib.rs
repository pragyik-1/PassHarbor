use tauri::Manager;

mod database;
mod encryptor;
pub mod error;
mod frontend_api;
mod master_key;
pub mod prelude;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            app.manage(state::AppState::default());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            frontend_api::unlock_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
