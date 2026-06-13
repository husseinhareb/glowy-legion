use tauri::State;

use crate::{app::state::AppState, domain::AppInfo};

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    state.app_info.clone()
}
