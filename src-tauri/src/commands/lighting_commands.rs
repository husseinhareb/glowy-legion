use tauri::State;

use crate::{app::state::AppState, domain::KeyboardState};

#[tauri::command]
pub fn get_keyboard_state(state: State<'_, AppState>) -> KeyboardState {
    state
        .lighting_service
        .get_keyboard_state()
        .unwrap_or_else(|_| KeyboardState::off())
}

#[tauri::command]
pub fn set_keyboard_state(
    app_state: State<'_, AppState>,
    state: KeyboardState,
) -> Result<KeyboardState, String> {
    app_state
        .lighting_service
        .set_keyboard_state(state)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn turn_backlight_off(state: State<'_, AppState>) -> Result<KeyboardState, String> {
    state
        .lighting_service
        .turn_backlight_off()
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn send_safe_test_payload(state: State<'_, AppState>) -> Result<KeyboardState, String> {
    state
        .lighting_service
        .send_safe_test_payload()
        .map_err(|error| error.to_user_message())
}
