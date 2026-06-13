use tauri::State;

use crate::{app::state::AppState, domain::DeviceInfo};

#[tauri::command]
pub fn detect_keyboard_device(state: State<'_, AppState>) -> DeviceInfo {
    state
        .device_service
        .detect_keyboard_device()
        .unwrap_or_else(|_| DeviceInfo::unsupported("Device detection failed", "mock"))
}
