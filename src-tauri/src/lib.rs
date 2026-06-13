mod app;
mod commands;
mod domain;
mod drivers;
mod infrastructure;
mod services;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app::bootstrap::build_app_state())
        .invoke_handler(tauri::generate_handler![
            commands::app_commands::get_app_info,
            commands::device_commands::detect_keyboard_device,
            commands::lighting_commands::get_keyboard_state,
            commands::lighting_commands::set_keyboard_state,
            commands::profile_commands::list_builtin_profiles,
            commands::profile_commands::apply_profile,
            commands::lighting_commands::turn_backlight_off,
            commands::lighting_commands::send_safe_test_payload,
            commands::diagnostics_commands::run_diagnostics,
            commands::permission_commands::preview_udev_rule,
            commands::permission_commands::probe_hid_access,
            commands::permission_commands::install_udev_rule_with_system_auth,
            commands::permission_commands::reload_udev_rules_with_system_auth,
            commands::permission_commands::remove_udev_rule_with_system_auth
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
