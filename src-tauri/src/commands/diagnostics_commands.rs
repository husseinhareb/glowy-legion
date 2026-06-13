use tauri::State;

use crate::{
    app::state::AppState,
    domain::{DeviceCapabilities, DeviceInfo, DiagnosticsReport},
    infrastructure::linux::dmi::read_dmi_info,
};

#[tauri::command]
pub fn run_diagnostics(state: State<'_, AppState>) -> DiagnosticsReport {
    let mut report = state
        .diagnostics_service
        .run_diagnostics()
        .unwrap_or_else(|error| fallback_diagnostics(error.to_user_message()));

    report
        .warnings
        .extend(state.app_info.configuration_warnings.clone());
    report
}

fn fallback_diagnostics(message: String) -> DiagnosticsReport {
    let dmi = read_dmi_info();
    let detected_device = DeviceInfo::unsupported("Diagnostics failed", "mock");

    DiagnosticsReport {
        os: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        backend_mode: "mock".to_string(),
        dmi_sys_vendor: dmi.sys_vendor,
        dmi_product_name: dmi.product_name,
        dmi_product_version: dmi.product_version,
        detected_device,
        hid_devices: Vec::new(),
        hid_interfaces: Vec::new(),
        eligible_rgb_interface_count: 0,
        hid_access_disabled_by_safety_flag: false,
        known_supported_lenovo_rgb_device: None,
        hid_device_opened: false,
        hid_access_probe: None,
        supported_effects: Vec::new(),
        unsupported_effects: Vec::new(),
        capabilities: DeviceCapabilities::unsupported(),
        real_hardware_backend_available: false,
        real_hardware_writes_enabled: false,
        dry_run_enabled: false,
        experimental_override_active: false,
        write_allowlist_source: "blocked".to_string(),
        requires_user_caution: false,
        likely_permission_issue: false,
        running_as_root: false,
        last_payload_hex: None,
        payload_preview: None,
        notes: Vec::new(),
        warnings: vec![message],
    }
}
