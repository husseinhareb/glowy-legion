use crate::domain::LightingEffect;
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AppError {
    #[error("device not found")]
    DeviceNotFound,
    #[error("unsupported device: {0}")]
    UnsupportedDevice(String),
    #[error("invalid brightness: {0}")]
    InvalidBrightness(u8),
    #[error("invalid speed: {0}")]
    InvalidSpeed(u8),
    #[error("invalid color: {0}")]
    InvalidColor(String),
    #[error("invalid effect {effect:?}: {reason}")]
    InvalidEffect {
        effect: LightingEffect,
        reason: String,
    },
    #[error("unsupported lighting effect: {0:?}")]
    UnsupportedEffect(LightingEffect),
    #[error("driver unavailable: {0}")]
    DriverUnavailable(String),
    #[error("HID error: {0}")]
    HidError(String),
    #[error("hardware backend is not implemented")]
    HardwareBackendNotImplemented,
    #[error("profile not found: {0}")]
    ProfileNotFound(String),
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl AppError {
    pub fn to_user_message(&self) -> String {
        match self {
            AppError::DeviceNotFound => "No keyboard backlight device was detected.".to_string(),
            AppError::UnsupportedDevice(device) => {
                format!("{device} is not supported by the active backend.")
            }
            AppError::InvalidBrightness(value) => {
                format!("Brightness must be between 0 and 100. Received {value}.")
            }
            AppError::InvalidSpeed(value) => {
                format!("Speed must be between 0 and 100. Received {value}.")
            }
            AppError::InvalidColor(message) => format!("Invalid RGB color: {message}."),
            AppError::InvalidEffect { effect, reason } => {
                format!("Effect {effect:?} cannot be applied: {reason}.")
            }
            AppError::UnsupportedEffect(effect) => {
                format!("Effect {effect:?} is not supported by the active backend.")
            }
            AppError::DriverUnavailable(message) => {
                format!("Keyboard driver is unavailable: {message}.")
            }
            AppError::HidError(message) => format!("HID backend error: {message}."),
            AppError::HardwareBackendNotImplemented => {
                "Real Lenovo hardware control is not implemented yet. Mock mode is active."
                    .to_string()
            }
            AppError::ProfileNotFound(profile_id) => {
                format!("Lighting profile '{profile_id}' was not found.")
            }
            AppError::StorageError(message) => format!("Storage error: {message}."),
            AppError::Unknown(message) => format!("Unexpected backend error: {message}."),
        }
    }
}

impl From<hidapi::HidError> for AppError {
    fn from(error: hidapi::HidError) -> Self {
        AppError::HidError(error.to_string())
    }
}
