use std::sync::Mutex;

use crate::{
    app::error::AppError,
    domain::{
        all_lighting_effects, supported_lighting_effects, DeviceInfo, DiagnosticsReport,
        HidDeviceSummary, KeyboardState,
    },
    drivers::keyboard_driver::KeyboardDriver,
    infrastructure::linux::dmi::read_dmi_info,
    utils::validation::{
        coerce_state_to_capabilities, normalize_keyboard_state, validate_keyboard_state,
    },
};

#[derive(Debug)]
pub struct MockKeyboardDriver {
    device: DeviceInfo,
    state: Mutex<KeyboardState>,
}

impl MockKeyboardDriver {
    pub fn new() -> Self {
        let device = mock_device_from_env();
        let state = mock_default_state(&device);

        Self {
            device,
            state: Mutex::new(state),
        }
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, KeyboardState>, AppError> {
        self.state
            .lock()
            .map_err(|_| AppError::DriverUnavailable("mock keyboard state lock failed".to_string()))
    }
}

impl Default for MockKeyboardDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardDriver for MockKeyboardDriver {
    fn backend_name(&self) -> &'static str {
        "mock"
    }

    fn detect_device(&self) -> Result<DeviceInfo, AppError> {
        Ok(self.device.clone())
    }

    fn get_state(&self) -> Result<KeyboardState, AppError> {
        Ok(self.lock_state()?.clone())
    }

    fn set_state(&self, state: KeyboardState) -> Result<KeyboardState, AppError> {
        let normalized = normalize_keyboard_state(state)?;
        let normalized = coerce_state_to_capabilities(normalized, &self.device.capabilities);
        validate_keyboard_state(&normalized, &self.device.capabilities)?;

        let mut current = self.lock_state()?;
        *current = normalized;
        Ok(current.clone())
    }

    fn turn_off(&self) -> Result<KeyboardState, AppError> {
        let mut current = self.lock_state()?;
        *current = KeyboardState::off();
        Ok(current.clone())
    }

    fn diagnostics(&self) -> Result<DiagnosticsReport, AppError> {
        let dmi = read_dmi_info();
        let supported_effects = supported_lighting_effects(&self.device.capabilities);
        let unsupported_effects = all_lighting_effects()
            .into_iter()
            .filter(|effect| !supported_effects.contains(effect))
            .collect();

        Ok(DiagnosticsReport {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            backend_mode: self.backend_name().to_string(),
            dmi_sys_vendor: dmi.sys_vendor,
            dmi_product_name: dmi.product_name,
            dmi_product_version: dmi.product_version,
            detected_device: self.device.clone(),
            hid_devices: Vec::<HidDeviceSummary>::new(),
            hid_interfaces: Vec::new(),
            eligible_rgb_interface_count: 0,
            hid_access_disabled_by_safety_flag: false,
            known_supported_lenovo_rgb_device: None,
            hid_device_opened: false,
            hid_access_probe: None,
            supported_effects,
            unsupported_effects,
            capabilities: self.device.capabilities.clone(),
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
            notes: vec![
                "Mock backend active. No hardware writes.".to_string(),
                "Use LEGIONGLOW_BACKEND=lenovo-hid-dry-run to test real HID detection without writes.".to_string(),
            ],
            warnings: Vec::new(),
        })
    }
}

fn mock_device_from_env() -> DeviceInfo {
    match std::env::var("LEGIONGLOW_MOCK_DEVICE") {
        Ok(value) if value.eq_ignore_ascii_case("loq") => DeviceInfo::mock_loq(),
        Ok(value) if value.eq_ignore_ascii_case("unsupported") => {
            DeviceInfo::unsupported("Unsupported Mock Laptop", "mock")
        }
        _ => DeviceInfo::mock_legion(),
    }
}

fn mock_default_state(device: &DeviceInfo) -> KeyboardState {
    let mut state = KeyboardState::default_static();

    if !device.capabilities.supports_secondary_color {
        state.secondary_color = None;
    }

    state
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{KeyboardState, LightingEffect},
        drivers::keyboard_driver::KeyboardDriver,
    };

    use super::MockKeyboardDriver;

    #[test]
    fn stores_mock_state_changes() {
        let driver = MockKeyboardDriver::new();
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Breathing;
        state.brightness = 44;

        let stored = driver.set_state(state).unwrap();

        assert_eq!(stored.effect, LightingEffect::Breathing);
        assert_eq!(driver.get_state().unwrap().brightness, 44);
    }

    #[test]
    fn turns_mock_backlight_off() {
        let driver = MockKeyboardDriver::new();

        let state = driver.turn_off().unwrap();

        assert_eq!(state.effect, LightingEffect::Off);
        assert!(!state.enabled);
    }
}
