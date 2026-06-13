#![allow(dead_code)]

pub struct LegionProtocol;

impl LegionProtocol {
    pub fn new() -> Self {
        Self
    }

    pub fn is_ready(&self) -> bool {
        // TODO: Implement model-specific Lenovo Legion protocol handling here after
        // validating whether a given laptop exposes keyboard RGB through sysfs,
        // hidraw, WMI, ACPI, or an existing kernel driver.
        false
    }
}
