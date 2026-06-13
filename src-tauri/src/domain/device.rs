use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceFamily {
    LenovoLegion,
    LenovoLoq,
    LenovoUnknown,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub supports_static: bool,
    pub supports_breathing: bool,
    pub supports_wave: bool,
    pub supports_reactive: bool,
    pub supports_rainbow: bool,
    pub supports_brightness: bool,
    pub supports_speed: bool,
    pub supports_direction: bool,
    pub supports_primary_color: bool,
    pub supports_secondary_color: bool,
    pub supports_zones: bool,
    pub zone_count: u8,
    pub supports_per_key_rgb: bool,
}

impl DeviceCapabilities {
    pub fn lenovo_4_zone_rgb() -> Self {
        Self {
            supports_static: true,
            supports_breathing: true,
            supports_wave: true,
            supports_reactive: false,
            supports_rainbow: true,
            supports_brightness: true,
            supports_speed: true,
            supports_direction: true,
            supports_primary_color: true,
            supports_secondary_color: false,
            supports_zones: true,
            zone_count: 4,
            supports_per_key_rgb: false,
        }
    }

    pub fn lenovo_lamp_array_4_zone_rgb() -> Self {
        Self {
            supports_static: true,
            supports_breathing: false,
            supports_wave: false,
            supports_reactive: false,
            supports_rainbow: false,
            supports_brightness: true,
            supports_speed: false,
            supports_direction: false,
            supports_primary_color: true,
            supports_secondary_color: false,
            supports_zones: true,
            zone_count: 4,
            supports_per_key_rgb: false,
        }
    }

    pub fn mock_legion() -> Self {
        Self {
            supports_static: true,
            supports_breathing: true,
            supports_wave: true,
            supports_reactive: true,
            supports_rainbow: true,
            supports_brightness: true,
            supports_speed: true,
            supports_direction: true,
            supports_primary_color: true,
            supports_secondary_color: true,
            supports_zones: true,
            zone_count: 4,
            supports_per_key_rgb: false,
        }
    }

    pub fn mock_loq() -> Self {
        Self {
            supports_static: true,
            supports_breathing: true,
            supports_wave: true,
            supports_reactive: false,
            supports_rainbow: false,
            supports_brightness: true,
            supports_speed: true,
            supports_direction: true,
            supports_primary_color: true,
            supports_secondary_color: false,
            supports_zones: true,
            zone_count: 4,
            supports_per_key_rgb: false,
        }
    }

    pub fn unsupported() -> Self {
        Self {
            supports_static: false,
            supports_breathing: false,
            supports_wave: false,
            supports_reactive: false,
            supports_rainbow: false,
            supports_brightness: false,
            supports_speed: false,
            supports_direction: false,
            supports_primary_color: false,
            supports_secondary_color: false,
            supports_zones: false,
            zone_count: 0,
            supports_per_key_rgb: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub vendor: String,
    pub product_name: String,
    pub family: DeviceFamily,
    pub supported: bool,
    pub backend: String,
    pub capabilities: DeviceCapabilities,
}

impl DeviceInfo {
    pub fn mock_legion() -> Self {
        Self {
            id: "mock-lenovo-legion-keyboard".to_string(),
            vendor: "Lenovo".to_string(),
            product_name: "Lenovo Legion 7 Mock".to_string(),
            family: DeviceFamily::LenovoLegion,
            supported: true,
            backend: "mock".to_string(),
            capabilities: DeviceCapabilities::mock_legion(),
        }
    }

    pub fn mock_loq() -> Self {
        Self {
            id: "mock-lenovo-loq-keyboard".to_string(),
            vendor: "Lenovo".to_string(),
            product_name: "Lenovo LOQ 15 Mock".to_string(),
            family: DeviceFamily::LenovoLoq,
            supported: true,
            backend: "mock".to_string(),
            capabilities: DeviceCapabilities::mock_loq(),
        }
    }

    pub fn unsupported(product_name: impl Into<String>, backend: impl Into<String>) -> Self {
        Self {
            id: "unsupported-device".to_string(),
            vendor: "Unknown".to_string(),
            product_name: product_name.into(),
            family: DeviceFamily::Unsupported,
            supported: false,
            backend: backend.into(),
            capabilities: DeviceCapabilities::unsupported(),
        }
    }
}
