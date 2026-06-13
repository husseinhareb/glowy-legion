use crate::{
    app::error::AppError,
    domain::{DeviceInfo, DiagnosticsReport, HidAccessProbe, KeyboardState, UdevRulePreview},
    infrastructure::linux::udev::unavailable_udev_rule_preview,
};

pub trait KeyboardDriver: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn detect_device(&self) -> Result<DeviceInfo, AppError>;
    fn get_state(&self) -> Result<KeyboardState, AppError>;
    fn set_state(&self, state: KeyboardState) -> Result<KeyboardState, AppError>;
    fn turn_off(&self) -> Result<KeyboardState, AppError>;
    fn diagnostics(&self) -> Result<DiagnosticsReport, AppError>;

    /// Send the lowest-risk real payload to verify hardware access. Only the
    /// Lenovo HID backend with real writes enabled implements this; everything
    /// else refuses.
    fn send_safe_test_payload(&self) -> Result<KeyboardState, AppError> {
        Err(AppError::UnsupportedDevice(
            "the safe test payload is only available on the Lenovo HID backend with real writes \
             enabled"
                .to_string(),
        ))
    }

    /// Preview (never install) a udev rule for the detected device.
    fn preview_udev_rule(&self) -> Result<UdevRulePreview, AppError> {
        Ok(unavailable_udev_rule_preview())
    }

    /// Manual-only access probe, run solely on an explicit user action (the
    /// "Probe HID access" button). Implementations may briefly open exactly one
    /// verified vendor-defined RGB-control interface and must drop the handle
    /// immediately. Never opens keyboard input interfaces, never sends feature
    /// reports, never enables writes, never changes lighting, and must never
    /// run automatically at startup or inside diagnostics.
    fn probe_hid_access(&self) -> Result<HidAccessProbe, AppError> {
        Err(AppError::UnsupportedDevice(
            "HID access probing is only available on the Lenovo HID backend".to_string(),
        ))
    }

    /// Raw `(vendor_id, product_id)` of the detected candidate, if any. Used by
    /// the permission flow to generate a rule from real device IDs (never from
    /// frontend input).
    fn detected_hid_ids(&self) -> Result<Option<(u16, u16)>, AppError> {
        Ok(None)
    }
}
