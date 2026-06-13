use serde::{Deserialize, Serialize};

use crate::domain::{device::DeviceCapabilities, device::DeviceInfo, lighting::LightingEffect};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HidDeviceSummary {
    pub vendor_id: String,
    pub product_id: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub path: Option<String>,
    pub known: bool,
    pub supported_for_writes: bool,
}

/// Safety classification of a single HID interface, built from passive
/// (sysfs-only) metadata. Interfaces are never opened to produce this.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HidInterfaceSummary {
    pub vendor_id: String,
    pub product_id: String,
    pub path: Option<String>,
    pub interface_number: Option<i32>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// Keyboard input interface — must never be opened by LegionGlow.
    pub is_keyboard_input: bool,
    pub is_consumer_control: bool,
    pub is_vendor_defined: bool,
    /// Standard HID LampArray lighting interface (usage page 0x59).
    pub is_lamp_array: bool,
    /// True only when this interface is safe to open for an explicit RGB probe.
    pub eligible_for_rgb_probe: bool,
    /// True when this interface carries a vendor-defined lighting collection and
    /// may receive feature reports from the ITE vendor write protocol. This is
    /// true even for composite interfaces that also contain keyboard input
    /// collections, because hidraw feature-report ioctls do not detach kernel
    /// drivers and cannot interfere with keyboard input.
    pub eligible_for_vendor_write: bool,
    pub safety_reason: String,
}

/// Categorized reason a HID device could not be opened. Used to turn the vague
/// hidapi error strings into actionable guidance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HidOpenFailureKind {
    PermissionDenied,
    DeviceBusy,
    BackendUnavailable,
    UnsupportedProduct,
    Unknown,
}

/// Decoded HID LampArrayAttributes feature report — a read-only summary of a
/// standard lighting interface (lamp count, kind, update interval). Reading it
/// never changes device state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LampArrayAttributesSummary {
    pub lamp_count: u16,
    pub lamp_array_kind: u32,
    /// Human-readable kind, e.g. "Keyboard".
    pub kind_label: String,
    pub min_update_interval_microseconds: u32,
    pub bounding_box_width_micrometers: u32,
    pub bounding_box_height_micrometers: u32,
    pub bounding_box_depth_micrometers: u32,
}

/// Structured result of probing a detected HID device for access readiness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HidAccessProbe {
    pub vendor_id: String,
    pub product_id: String,
    pub label: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub path_available: bool,
    pub can_open: bool,
    pub failure_kind: Option<HidOpenFailureKind>,
    pub raw_error: Option<String>,
    pub user_message: String,
    pub recommended_action: String,
    /// Present when the probed interface is a LampArray and its attributes
    /// report was read successfully. Read-only; no writes are involved.
    pub lamp_array_attributes: Option<LampArrayAttributesSummary>,
}

/// A non-installing preview of a udev rule that would grant the current user
/// access to the detected HID device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdevRulePreview {
    pub available: bool,
    pub vendor_id: String,
    pub product_id: String,
    pub rule: String,
    pub filename: String,
    pub explanation: String,
    pub install_commands: Vec<String>,
    pub reload_commands: Vec<String>,
    pub warnings: Vec<String>,
}

/// Decoded view of the most recent HID feature report payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HidPayloadPreview {
    pub length: usize,
    pub hex: String,
    pub header_bytes: Vec<String>,
    pub effect_byte: String,
    pub speed_byte: String,
    pub brightness_byte: String,
    pub zone_bytes: Vec<String>,
    pub direction_bytes: Vec<String>,
    pub decoded_effect: String,
}

/// Where the active real-write decision came from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WriteAllowlistSource {
    BuiltIn,
    EnvironmentOverride,
    Blocked,
}

impl WriteAllowlistSource {
    pub fn as_str(self) -> &'static str {
        match self {
            WriteAllowlistSource::BuiltIn => "built-in",
            WriteAllowlistSource::EnvironmentOverride => "environment override",
            WriteAllowlistSource::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticsReport {
    pub os: String,
    pub architecture: String,
    pub backend_mode: String,
    pub dmi_sys_vendor: Option<String>,
    pub dmi_product_name: Option<String>,
    pub dmi_product_version: Option<String>,
    pub detected_device: DeviceInfo,
    pub hid_devices: Vec<HidDeviceSummary>,
    /// Per-interface safety classification from passive sysfs metadata.
    pub hid_interfaces: Vec<HidInterfaceSummary>,
    /// Number of interfaces considered safe to open for an explicit RGB probe.
    /// The probe button is enabled only when this is exactly 1.
    pub eligible_rgb_interface_count: usize,
    /// True when LEGIONGLOW_DISABLE_HID blocks all active HID access.
    pub hid_access_disabled_by_safety_flag: bool,
    pub known_supported_lenovo_rgb_device: Option<HidDeviceSummary>,
    pub hid_device_opened: bool,
    pub hid_access_probe: Option<HidAccessProbe>,
    pub supported_effects: Vec<LightingEffect>,
    pub unsupported_effects: Vec<LightingEffect>,
    pub capabilities: DeviceCapabilities,
    pub real_hardware_backend_available: bool,
    pub real_hardware_writes_enabled: bool,
    pub dry_run_enabled: bool,
    pub experimental_override_active: bool,
    pub write_allowlist_source: String,
    pub requires_user_caution: bool,
    pub likely_permission_issue: bool,
    pub running_as_root: bool,
    pub last_payload_hex: Option<String>,
    pub payload_preview: Option<HidPayloadPreview>,
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
}
