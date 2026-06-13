use std::sync::Arc;

use crate::{
    app::error::AppError,
    domain::{DeviceInfo, KeyboardState},
    drivers::keyboard_driver::KeyboardDriver,
    utils::validation::{
        coerce_state_to_capabilities, ensure_device_supported, normalize_keyboard_state,
        validate_keyboard_state,
    },
};

#[derive(Clone)]
pub struct LightingService {
    driver: Arc<dyn KeyboardDriver>,
}

impl LightingService {
    pub fn new(driver: Arc<dyn KeyboardDriver>) -> Self {
        Self { driver }
    }

    pub fn detect_device(&self) -> Result<DeviceInfo, AppError> {
        self.driver.detect_device()
    }

    pub fn get_keyboard_state(&self) -> Result<KeyboardState, AppError> {
        self.driver.get_state()
    }

    pub fn set_keyboard_state(&self, state: KeyboardState) -> Result<KeyboardState, AppError> {
        let device = self.driver.detect_device()?;
        ensure_device_supported(&device)?;

        let normalized = normalize_keyboard_state(state)?;
        let coerced = coerce_state_to_capabilities(normalized, &device.capabilities);
        validate_keyboard_state(&coerced, &device.capabilities)?;

        self.driver.set_state(coerced)
    }

    pub fn turn_backlight_off(&self) -> Result<KeyboardState, AppError> {
        self.driver.turn_off()
    }

    pub fn send_safe_test_payload(&self) -> Result<KeyboardState, AppError> {
        self.driver.send_safe_test_payload()
    }
}
