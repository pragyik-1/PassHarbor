use std::sync::Arc;

use tauri::{Manager, State};

use crate::AppState;

pub fn resolve_app_state(app: &tauri::AppHandle) -> State<'_, AppState> {
    app.state::<AppState>()
}