use std::sync::Arc;

use crate::{
    app::state::AppState,
    domain::AppInfo,
    drivers::{
        keyboard_driver::KeyboardDriver, lenovo::lenovo_keyboard_driver::LenovoKeyboardDriver,
        mock::MockKeyboardDriver,
    },
    infrastructure::{
        logging,
        storage::{
            profile_repository::{BuiltinProfileRepository, ProfileRepository},
            settings_repository::{
                BackendMode, BackendSelection, ExperimentalAllowlist, SettingsRepository,
            },
        },
    },
    services::{
        device_service::DeviceService, diagnostics_service::DiagnosticsService,
        lighting_service::LightingService, permission_service::PermissionService,
        profile_service::ProfileService,
    },
};

pub fn build_app_state() -> AppState {
    logging::initialize_logging();

    let profile_repository: Arc<dyn ProfileRepository> = Arc::new(BuiltinProfileRepository::new());
    let settings_repository = SettingsRepository::new();
    let selection = settings_repository.backend_selection();
    let experimental_allow = settings_repository.experimental_allowlist();
    let hid_access_disabled = settings_repository.hid_access_disabled();
    let (backend_mode, keyboard_driver, configuration_warnings) =
        build_keyboard_driver(selection, experimental_allow, hid_access_disabled);
    let real_hardware_writes_enabled = backend_mode == BackendMode::LenovoHid.as_str();
    let requires_user_caution =
        matches!(backend_mode.as_str(), "lenovo-hid-dry-run" | "lenovo-hid");

    let device_service = Arc::new(DeviceService::new(keyboard_driver.clone()));
    let lighting_service = Arc::new(LightingService::new(keyboard_driver.clone()));
    let profile_service = Arc::new(ProfileService::new(
        lighting_service.clone(),
        profile_repository,
    ));
    let permission_service = Arc::new(PermissionService::new(keyboard_driver.clone()));
    let diagnostics_service = Arc::new(DiagnosticsService::new(keyboard_driver));

    AppState {
        app_info: AppInfo {
            name: "LegionGlow".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            backend_mode,
            real_hardware_writes_enabled,
            requires_user_caution,
            configuration_warnings,
        },
        device_service,
        lighting_service,
        profile_service,
        diagnostics_service,
        permission_service,
    }
}

fn build_keyboard_driver(
    selection: BackendSelection,
    experimental_allow: ExperimentalAllowlist,
    hid_access_disabled: bool,
) -> (String, Arc<dyn KeyboardDriver>, Vec<String>) {
    let mut warnings = selection.warnings;
    warnings.extend(experimental_allow.warnings.clone());

    if hid_access_disabled {
        warnings.push("HID access disabled by safety flag.".to_string());
    }

    match selection.mode {
        BackendMode::Mock => (
            BackendMode::Mock.as_str().to_string(),
            Arc::new(MockKeyboardDriver::new()),
            warnings,
        ),
        BackendMode::LenovoHidDryRun | BackendMode::LenovoHid => {
            // Construction is passive: it stores configuration only and never
            // initializes a HID library, opens a device, or probes access.
            let driver =
                LenovoKeyboardDriver::new(selection.mode, experimental_allow, hid_access_disabled);
            (
                selection.mode.as_str().to_string(),
                Arc::new(driver),
                warnings,
            )
        }
    }
}
