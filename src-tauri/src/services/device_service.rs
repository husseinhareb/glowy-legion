use std::sync::Arc;

use crate::{app::error::AppError, domain::DeviceInfo, drivers::keyboard_driver::KeyboardDriver};

#[derive(Clone)]
pub struct DeviceService {
    driver: Arc<dyn KeyboardDriver>,
}

impl DeviceService {
    pub fn new(driver: Arc<dyn KeyboardDriver>) -> Self {
        Self { driver }
    }

    pub fn detect_keyboard_device(&self) -> Result<DeviceInfo, AppError> {
        self.driver.detect_device()
    }
}
