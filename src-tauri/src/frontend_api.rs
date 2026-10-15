use crate::master_key::MasterKey;
use std::sync::Mutex;

#[tauri::command]
pub fn set_master_key_cmd(
    state: tauri::State<'_, Mutex<MasterKey>>,
    key: [u8; 32],
) -> Result<(), String> {
    let mut master_key = state.lock().unwrap();
    master_key.set_key(key);
    Ok(())
}
