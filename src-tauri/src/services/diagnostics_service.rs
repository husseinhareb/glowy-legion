use std::sync::Arc;

use crate::{
    app::error::AppError, domain::DiagnosticsReport, drivers::keyboard_driver::KeyboardDriver,
};

#[derive(Clone)]
pub struct DiagnosticsService {
    driver: Arc<dyn KeyboardDriver>,
}

impl DiagnosticsService {
    pub fn new(driver: Arc<dyn KeyboardDriver>) -> Self {
        Self { driver }
    }

    pub fn run_diagnostics(&self) -> Result<DiagnosticsReport, AppError> {
        self.driver.diagnostics()
    }
}
