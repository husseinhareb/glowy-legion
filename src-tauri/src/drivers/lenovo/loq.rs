#![allow(dead_code)]

pub struct LoqProtocol;

impl LoqProtocol {
    pub fn new() -> Self {
        Self
    }

    pub fn is_ready(&self) -> bool {
        // TODO: Implement Lenovo LOQ keyboard lighting protocol handling here.
        // LOQ capabilities may be more limited than Legion capabilities and should
        // be reported through DeviceCapabilities before controls are enabled.
        false
    }
}
