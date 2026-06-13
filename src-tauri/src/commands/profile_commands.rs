use tauri::State;

use crate::{
    app::state::AppState,
    domain::{KeyboardState, LightingProfile},
};

#[tauri::command]
pub fn list_builtin_profiles(state: State<'_, AppState>) -> Vec<LightingProfile> {
    state
        .profile_service
        .list_builtin_profiles()
        .unwrap_or_default()
}

#[tauri::command]
pub fn apply_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<KeyboardState, String> {
    state
        .profile_service
        .apply_profile(&profile_id)
        .map_err(|error| error.to_user_message())
}
