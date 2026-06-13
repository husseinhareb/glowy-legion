use tauri::State;

use crate::{
    app::state::AppState,
    domain::{HidAccessProbe, PermissionSetupResult, UdevRulePreview},
};

#[tauri::command]
pub fn preview_udev_rule(state: State<'_, AppState>) -> Result<UdevRulePreview, String> {
    state
        .permission_service
        .preview_udev_rule()
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn probe_hid_access(state: State<'_, AppState>) -> Result<HidAccessProbe, String> {
    state
        .permission_service
        .probe_hid_access()
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn install_udev_rule_with_system_auth(
    state: State<'_, AppState>,
    password: String,
) -> Result<PermissionSetupResult, String> {
    state
        .permission_service
        .install_udev_rule(&password)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn reload_udev_rules_with_system_auth(
    state: State<'_, AppState>,
    password: String,
) -> Result<PermissionSetupResult, String> {
    state
        .permission_service
        .reload_udev_rules(&password)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn remove_udev_rule_with_system_auth(
    state: State<'_, AppState>,
    password: String,
) -> Result<PermissionSetupResult, String> {
    state
        .permission_service
        .remove_udev_rule(&password)
        .map_err(|error| error.to_user_message())
}
