use tauri::Manager;

mod database;
pub mod error;
mod frontend_api;
mod keychain;
mod master_key;
pub mod prelude;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let vault_dir = app.path().app_data_dir()?.join("vaults");
            app.manage(state::AppState::new(database::VaultStore::new(vault_dir)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_api::unlock_cmd,
            frontend_api::unlock_with_keychain_cmd,
            frontend_api::enable_keychain_cmd,
            frontend_api::disable_keychain_cmd,
            frontend_api::keychain_available_cmd,
            frontend_api::lock_cmd,
            frontend_api::get_session_cmd,
            frontend_api::list_vaults_cmd,
            frontend_api::set_default_vault_cmd,
            frontend_api::create_vault_cmd,
            frontend_api::add_entry_cmd,
            frontend_api::list_entries_cmd,
            frontend_api::query_entries_cmd,
            frontend_api::get_entry_cmd,
            frontend_api::update_entry_cmd,
            frontend_api::delete_entry_cmd,
            frontend_api::set_password_cmd,
            frontend_api::add_custom_kind_cmd,
            frontend_api::remove_custom_kind_cmd,
            frontend_api::reset_custom_kinds_cmd,
            frontend_api::list_custom_kinds_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
