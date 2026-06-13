use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub backend_mode: String,
    pub real_hardware_writes_enabled: bool,
    pub requires_user_caution: bool,
    pub configuration_warnings: Vec<String>,
}
