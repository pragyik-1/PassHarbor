use zeroize::Zeroize;

use crate::prelude::*;
use crate::{encryptor, error::Error, error::emit_error, utils::resolve_app_state};

#[tauri::command]
pub fn set_master_key_cmd(
    app: tauri::AppHandle,
    key: [u8; 32],
) -> IPCResult<()> {
    let app_state = resolve_app_state(&app);

    let mut master_key = app_state.master_key.lock().unwrap();
    master_key.set_key(key);

    let mut db_key = app_state.db_key.lock().unwrap();
    let key = master_key.derive_db_key().map_err(|e| {
        let message = format!("Failed to derive db_key: {e}");
        emit_error(&app, &Error::Generic(message.clone()));
        message
    })?;
    *db_key = Some(key);

    let mut enc_key = app_state.enc_key.lock().unwrap();
    let key = master_key.derive_enc_key().map_err(|e| {
        let message = format!("Failed to derive enc_key: {e}");
        emit_error(&app, &Error::Generic(message.clone()));
        message
    })?;
    *enc_key = Some(key);

    Ok(())
}

#[tauri::command]
pub fn clear_master_key_cmd(app: tauri::AppHandle) -> IPCResult<()> {
    let app_state = resolve_app_state(&app);

    let mut master_key = app_state.master_key.lock().unwrap();
    master_key.zeroize();

    let mut db_key = app_state.db_key.lock().unwrap();
    *db_key = None;

    let mut enc_key = app_state.enc_key.lock().unwrap();
    *enc_key = None;

    Ok(())
}