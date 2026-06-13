//! Lenovo HID keyboard driver.
//!
//! SAFETY MODEL (do not regress):
//! - The constructor and all startup paths are passive: DMI + hidraw sysfs
//!   metadata only. No HID library is initialized, no device is opened, no
//!   kernel driver is detached.
//! - HID devices are opened only inside explicit, user-triggered active
//!   operations (the manual access probe and real writes), only through the
//!   single RGB-control interface (vendor-defined or HID LampArray) selected
//!   by `interface_filter`, and the handle is dropped immediately afterwards.
//! - The vendor write protocol is never sent to a LampArray interface. Devices
//!   that use LampArray go through the standard LampArray SET_FEATURE path.
//! - Keyboard input interfaces are never opened. A VID/PID match alone is
//!   never sufficient to open anything.
//! - `LEGIONGLOW_DISABLE_HID=1` blocks every active operation.

use std::sync::Mutex;

use crate::{
    app::error::AppError,
    domain::{
        all_lighting_effects, supported_lighting_effects, DeviceInfo, DiagnosticsReport,
        HidAccessProbe, HidDeviceSummary, HidInterfaceSummary, HidOpenFailureKind, KeyboardState,
        LampArrayAttributesSummary, UdevRulePreview, WriteAllowlistSource,
    },
    drivers::{
        keyboard_driver::KeyboardDriver,
        lenovo::{
            detection::{
                detect_lenovo_rgb_device_from_metadata,
                list_possible_lenovo_hid_devices_from_metadata, unsupported_device_from_dmi,
                LenovoHidDeviceInfo, LenovoWriteProtocol,
            },
            hid_probe::build_hid_access_probe,
            interface_filter::{
                classify_hid_interface, select_probe_target, select_vendor_write_target,
            },
            lamp_array::{build_lamp_array_update_reports, parse_lamp_array_attributes_report},
            protocol::{
                build_feature_report, build_safe_test_state, decode_payload_preview,
                FEATURE_REPORT_LEN,
            },
        },
    },
    infrastructure::{
        linux::{
            dmi::{read_dmi_info, DmiInfo},
            hidraw::{enumerate_hidraw_interfaces, HidrawInterfaceMetadata},
            process::is_running_as_root,
            udev::{build_udev_rule_preview, unavailable_udev_rule_preview},
        },
        storage::settings_repository::{BackendMode, ExperimentalAllowlist},
    },
    utils::validation::{
        coerce_state_to_capabilities, ensure_device_supported, normalize_keyboard_state,
        validate_keyboard_state,
    },
};

/// Ephemeral handle to an opened HID device. Dropped as soon as the single
/// active operation completes; never stored in app state.
pub trait HidHandle {
    fn send_feature_report(&self, payload: &[u8]) -> Result<(), AppError>;

    /// GET_FEATURE for `report_id`. Read-only: never changes device state.
    /// Returns the report bytes including the leading report ID byte.
    fn read_feature_report(&self, report_id: u8) -> Result<Vec<u8>, AppError>;
}

/// The only way this driver opens HID devices. Production uses hidapi with the
/// Linux hidraw backend; tests inject counting/failing fakes.
pub trait HidOpener: Send + Sync {
    fn open_path(&self, path: &str) -> Result<Box<dyn HidHandle>, AppError>;
}

/// Production opener. The hidapi context is created lazily per call (hidraw
/// backend, without enumeration) so that nothing HID-related runs at startup.
pub struct HidapiHidOpener;

struct HidapiHandle {
    device: hidapi::HidDevice,
}

impl HidHandle for HidapiHandle {
    fn send_feature_report(&self, payload: &[u8]) -> Result<(), AppError> {
        self.device
            .send_feature_report(payload)
            .map_err(AppError::from)
    }

    fn read_feature_report(&self, report_id: u8) -> Result<Vec<u8>, AppError> {
        // Generous buffer: hidraw GET_FEATURE truncates to the actual report
        // size and returns the number of bytes read.
        let mut buffer = vec![0u8; 4096];
        buffer[0] = report_id;
        let length = self
            .device
            .get_feature_report(&mut buffer)
            .map_err(AppError::from)?;
        buffer.truncate(length);
        Ok(buffer)
    }
}

impl HidOpener for HidapiHidOpener {
    fn open_path(&self, path: &str) -> Result<Box<dyn HidHandle>, AppError> {
        // Deliberately skip device discovery: this opener targets exactly one
        // pre-filtered hidraw path and must not touch anything else.
        #[allow(deprecated)]
        let api = hidapi::HidApi::new_without_enumerate().map_err(AppError::from)?;
        let c_path = std::ffi::CString::new(path)
            .map_err(|_| AppError::HidError(format!("invalid device path: {path}")))?;
        let device = api.open_path(&c_path).map_err(AppError::from)?;
        Ok(Box::new(HidapiHandle { device }))
    }
}

type InterfaceSource = Box<dyn Fn() -> Vec<HidrawInterfaceMetadata> + Send + Sync>;

pub struct LenovoKeyboardDriver {
    mode: BackendMode,
    experimental_allow: ExperimentalAllowlist,
    hid_access_disabled: bool,
    opener: Box<dyn HidOpener>,
    interface_source: InterfaceSource,
    current_state: Mutex<KeyboardState>,
    last_payload: Mutex<Option<Vec<u8>>>,
}

/// Passive view of the world, recomputed on demand from sysfs. Building this
/// never opens or claims anything.
struct PassiveSnapshot {
    dmi: DmiInfo,
    devices: Vec<LenovoHidDeviceInfo>,
    interfaces: Vec<HidrawInterfaceMetadata>,
    candidate: Option<LenovoHidDeviceInfo>,
}

impl LenovoKeyboardDriver {
    pub fn new(
        mode: BackendMode,
        experimental_allow: ExperimentalAllowlist,
        hid_access_disabled: bool,
    ) -> Self {
        Self::with_dependencies(
            mode,
            experimental_allow,
            hid_access_disabled,
            Box::new(HidapiHidOpener),
            Box::new(enumerate_hidraw_interfaces),
        )
    }

    /// Constructor with injectable HID access points. The constructor itself
    /// touches neither: detection runs lazily per call.
    pub fn with_dependencies(
        mode: BackendMode,
        experimental_allow: ExperimentalAllowlist,
        hid_access_disabled: bool,
        opener: Box<dyn HidOpener>,
        interface_source: InterfaceSource,
    ) -> Self {
        Self {
            mode,
            experimental_allow,
            hid_access_disabled,
            opener,
            interface_source,
            current_state: Mutex::new(default_lenovo_state()),
            last_payload: Mutex::new(None),
        }
    }

    fn passive_snapshot(&self) -> PassiveSnapshot {
        let dmi = read_dmi_info();
        let interfaces = if self.hid_access_disabled {
            // Safety flag: fall back to DMI-only detection. hidraw metadata
            // enumeration is also sysfs-only, but the flag means "touch as
            // little as possible", so skip it entirely.
            Vec::new()
        } else {
            (self.interface_source)()
        };
        let devices = list_possible_lenovo_hid_devices_from_metadata(&interfaces, &dmi);
        // A device explicitly named by the experimental allowlist also counts
        // as the candidate — the hard pending-verification block still applies
        // to it later, before any write.
        let candidate = detect_lenovo_rgb_device_from_metadata(
            &interfaces,
            &dmi,
            self.mode == BackendMode::LenovoHidDryRun,
        )
        .ok()
        .or_else(|| {
            devices
                .iter()
                .find(|device| {
                    self.experimental_allow
                        .contains(device.vendor_id, device.product_id)
                })
                .cloned()
        });

        PassiveSnapshot {
            dmi,
            devices,
            interfaces,
            candidate,
        }
    }

    fn interface_summaries(&self, snapshot: &PassiveSnapshot) -> Vec<HidInterfaceSummary> {
        snapshot
            .interfaces
            .iter()
            .map(|interface| {
                let is_candidate = snapshot
                    .candidate
                    .as_ref()
                    .map(|candidate| {
                        candidate.vendor_id == interface.vendor_id
                            && candidate.product_id == interface.product_id
                    })
                    .unwrap_or(false);
                classify_hid_interface(interface, is_candidate)
            })
            .collect()
    }

    fn writes_enabled(&self) -> bool {
        self.mode.real_hardware_writes_enabled()
    }

    fn dry_run_enabled(&self) -> bool {
        self.mode == BackendMode::LenovoHidDryRun
    }

    fn experimental_allows(&self, device: &LenovoHidDeviceInfo) -> bool {
        self.experimental_allow
            .contains(device.vendor_id, device.product_id)
    }

    fn experimental_override_active(&self, snapshot: &PassiveSnapshot) -> bool {
        snapshot
            .candidate
            .as_ref()
            .map(|device| !device.supported_for_writes && self.experimental_allows(device))
            .unwrap_or(false)
    }

    fn write_allowlist_source(&self, snapshot: &PassiveSnapshot) -> WriteAllowlistSource {
        match snapshot.candidate.as_ref() {
            Some(device) if device.supported_for_writes => WriteAllowlistSource::BuiltIn,
            Some(device) if self.experimental_allows(device) => {
                WriteAllowlistSource::EnvironmentOverride
            }
            _ => WriteAllowlistSource::Blocked,
        }
    }

    /// Whether real writes would actually reach the detected device.
    fn effective_writes_enabled(&self, snapshot: &PassiveSnapshot) -> bool {
        if self.hid_access_disabled || !self.writes_enabled() {
            return false;
        }

        let device = match snapshot.candidate.as_ref() {
            Some(device) if device.supported_for_writes || self.experimental_allows(device) => {
                device
            }
            _ => return false,
        };
        let protocol = self.write_protocol_for_device(device);
        let summaries = self.interface_summaries(snapshot);

        match protocol {
            LenovoWriteProtocol::IteVendor => select_vendor_write_target(&summaries).is_ok(),
            LenovoWriteProtocol::HidLampArray => {
                let target = match select_probe_target(&summaries) {
                    Ok(target) => target,
                    Err(_) => return false,
                };
                if !target.is_lamp_array {
                    return false;
                }
                target
                    .path
                    .as_deref()
                    .and_then(|path| {
                        snapshot
                            .interfaces
                            .iter()
                            .find(|interface| interface.dev_path == path)
                    })
                    .map(|interface| {
                        let reports = interface.lamp_array_reports;
                        reports.control.is_some() && reports.range_update.is_some()
                    })
                    .unwrap_or(false)
            }
        }
    }

    fn device_is_supported_for_active_mode(&self, device: &LenovoHidDeviceInfo) -> bool {
        device.supported_for_writes
            || (self.dry_run_enabled() && device.dry_run_protocol_candidate)
            || self.experimental_allows(device)
    }

    fn device_info_from_snapshot(&self, snapshot: &PassiveSnapshot) -> DeviceInfo {
        snapshot
            .candidate
            .as_ref()
            .map(|device| {
                device.to_device_info(
                    self.backend_name(),
                    self.device_is_supported_for_active_mode(device),
                )
            })
            .unwrap_or_else(|| unsupported_device_from_dmi(self.backend_name()))
    }

    fn ensure_active_access_allowed(&self) -> Result<(), AppError> {
        if self.hid_access_disabled {
            return Err(AppError::DriverUnavailable(
                "HID access disabled by safety flag (LEGIONGLOW_DISABLE_HID=1)".to_string(),
            ));
        }
        Ok(())
    }

    fn save_current_state(&self, state: KeyboardState) -> Result<KeyboardState, AppError> {
        let mut current = self
            .current_state
            .lock()
            .map_err(|_| AppError::DriverUnavailable("Lenovo HID state lock failed".to_string()))?;
        *current = state;
        Ok(current.clone())
    }

    fn save_last_payload(&self, payload: &[u8]) -> Result<(), AppError> {
        let mut last_payload = self.last_payload.lock().map_err(|_| {
            AppError::DriverUnavailable("Lenovo HID payload byte lock failed".to_string())
        })?;
        *last_payload = Some(payload.to_vec());
        Ok(())
    }

    fn last_payload(&self) -> Option<Vec<u8>> {
        self.last_payload
            .lock()
            .ok()
            .and_then(|payload| payload.clone())
    }

    /// Read-only LampArray attributes read through an already-open handle.
    /// Best effort: a failed read never fails the probe itself.
    fn read_lamp_array_attributes(
        &self,
        snapshot: &PassiveSnapshot,
        path: &str,
        handle: &dyn HidHandle,
    ) -> Option<LampArrayAttributesSummary> {
        let report_id = snapshot
            .interfaces
            .iter()
            .find(|interface| interface.dev_path == path)
            .and_then(|interface| interface.lamp_array_reports.attributes)?;
        let bytes = handle.read_feature_report(report_id).ok()?;
        parse_lamp_array_attributes_report(&bytes).ok()
    }

    fn write_protocol_for_device(&self, device: &LenovoHidDeviceInfo) -> LenovoWriteProtocol {
        device
            .write_protocol
            .unwrap_or(LenovoWriteProtocol::IteVendor)
    }

    fn lamp_array_report_ids_for_path(
        &self,
        snapshot: &PassiveSnapshot,
        path: &str,
    ) -> Result<crate::infrastructure::linux::hidraw::LampArrayReportIds, AppError> {
        snapshot
            .interfaces
            .iter()
            .find(|interface| interface.dev_path == path)
            .map(|interface| interface.lamp_array_reports)
            .ok_or_else(|| {
                AppError::UnsupportedDevice(
                    "selected LampArray interface metadata is unavailable".to_string(),
                )
            })
    }

    /// Build (and in real-write mode, send) a feature report for `state`.
    ///
    /// Dry-run never opens anything. Real writes open the single eligible
    /// RGB-control interface, send one report, and drop the handle.
    fn apply_state_internal(&self, state: KeyboardState) -> Result<KeyboardState, AppError> {
        let snapshot = self.passive_snapshot();
        let device_info = self.device_info_from_snapshot(&snapshot);
        ensure_device_supported(&device_info)?;

        let normalized = normalize_keyboard_state(state)?;
        let normalized = coerce_state_to_capabilities(normalized, &device_info.capabilities);
        validate_keyboard_state(&normalized, &device_info.capabilities)?;

        let device = snapshot
            .candidate
            .as_ref()
            .ok_or(AppError::DeviceNotFound)?;
        let protocol = self.write_protocol_for_device(device);
        let summaries = self.interface_summaries(&snapshot);
        let target = match protocol {
            LenovoWriteProtocol::HidLampArray => {
                Some(select_probe_target(&summaries).map_err(AppError::UnsupportedDevice)?)
            }
            LenovoWriteProtocol::IteVendor if self.writes_enabled() => {
                Some(select_vendor_write_target(&summaries).map_err(AppError::UnsupportedDevice)?)
            }
            LenovoWriteProtocol::IteVendor => None,
        };

        let vendor_payload = match protocol {
            LenovoWriteProtocol::IteVendor => {
                let payload = build_feature_report(&normalized, &device_info.capabilities)?;
                self.save_last_payload(&payload)?;
                Some(payload)
            }
            LenovoWriteProtocol::HidLampArray => None,
        };

        let lamp_array_target = match (protocol, target) {
            (LenovoWriteProtocol::HidLampArray, Some(target)) => {
                if !target.is_lamp_array {
                    return Err(AppError::UnsupportedDevice(
                        "this device is configured for HID LampArray writes, but the selected \
                         interface is not a LampArray"
                            .to_string(),
                    ));
                }
                let path = target.path.as_deref().ok_or_else(|| {
                    AppError::HidError("eligible interface has no path".to_string())
                })?;
                let report_ids = self.lamp_array_report_ids_for_path(&snapshot, path)?;
                // Diagnostics preview only. The device's real lamp count is read
                // from the hardware in the write path below; here we use the zone
                // count as a dry-run-safe fallback so a preview never opens the
                // device.
                let preview = build_lamp_array_update_reports(
                    &normalized,
                    &device_info.capabilities,
                    &report_ids,
                    device_info.capabilities.zone_count as u16,
                )?;
                let mut preview_bytes = preview.control.clone();
                for update in &preview.updates {
                    preview_bytes.extend_from_slice(update);
                }
                self.save_last_payload(&preview_bytes)?;
                Some((target, report_ids))
            }
            _ => None,
        };

        if self.writes_enabled() {
            self.ensure_active_access_allowed()?;

            if !self.effective_writes_enabled(&snapshot) {
                return Err(AppError::UnsupportedDevice(
                    "real writes are blocked for the detected product. Enable it via the \
                     experimental allowlist before writing"
                        .to_string(),
                ));
            }

            match protocol {
                LenovoWriteProtocol::IteVendor => {
                    let target = target.ok_or_else(|| {
                        AppError::UnsupportedDevice(
                            "No safe RGB-control HID interface was identified. LegionGlow will \
                             not open this device."
                                .to_string(),
                        )
                    })?;
                    let path = target.path.clone().ok_or_else(|| {
                        AppError::HidError("eligible interface has no path".to_string())
                    })?;
                    let payload = vendor_payload.as_ref().ok_or_else(|| {
                        AppError::DriverUnavailable("vendor payload was not built".to_string())
                    })?;

                    // Open, send one report, drop the handle immediately.
                    let handle = self.opener.open_path(&path)?;
                    handle.send_feature_report(payload)?;
                    drop(handle);
                }
                LenovoWriteProtocol::HidLampArray => {
                    let (target, report_ids) = lamp_array_target.ok_or_else(|| {
                        AppError::DriverUnavailable("LampArray target was not resolved".to_string())
                    })?;
                    let path = target.path.clone().ok_or_else(|| {
                        AppError::HidError("eligible interface has no path".to_string())
                    })?;

                    let handle = self.opener.open_path(&path)?;
                    // The keyboard's actual lamp count drives how the four zones
                    // map onto the physical lamps. Reading it (read-only) is what
                    // lets a single update cover the whole keyboard instead of
                    // only the first few lamps.
                    let lamp_count =
                        match self.read_lamp_array_attributes(&snapshot, &path, handle.as_ref()) {
                            Some(attributes) => {
                                if attributes.lamp_count
                                    < device_info.capabilities.zone_count as u16
                                {
                                    return Err(AppError::UnsupportedDevice(format!(
                                        "LampArray reports {} lamps, but LegionGlow needs {} zones",
                                        attributes.lamp_count, device_info.capabilities.zone_count
                                    )));
                                }
                                attributes.lamp_count
                            }
                            None => device_info.capabilities.zone_count as u16,
                        };

                    let reports = build_lamp_array_update_reports(
                        &normalized,
                        &device_info.capabilities,
                        &report_ids,
                        lamp_count,
                    )?;
                    handle.send_feature_report(&reports.control)?;
                    for update in &reports.updates {
                        handle.send_feature_report(update)?;
                    }
                    drop(handle);
                }
            }
        }

        self.save_current_state(normalized)
    }
}

impl KeyboardDriver for LenovoKeyboardDriver {
    fn backend_name(&self) -> &'static str {
        self.mode.as_str()
    }

    fn detect_device(&self) -> Result<DeviceInfo, AppError> {
        let snapshot = self.passive_snapshot();
        Ok(self.device_info_from_snapshot(&snapshot))
    }

    fn get_state(&self) -> Result<KeyboardState, AppError> {
        self.current_state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| AppError::DriverUnavailable("Lenovo HID state lock failed".to_string()))
    }

    fn set_state(&self, state: KeyboardState) -> Result<KeyboardState, AppError> {
        self.apply_state_internal(state)
    }

    fn turn_off(&self) -> Result<KeyboardState, AppError> {
        self.set_state(KeyboardState::off())
    }

    fn send_safe_test_payload(&self) -> Result<KeyboardState, AppError> {
        if !self.writes_enabled() {
            return Err(AppError::UnsupportedDevice(
                "the safe test payload requires the lenovo-hid backend with real writes enabled"
                    .to_string(),
            ));
        }
        self.ensure_active_access_allowed()?;

        self.apply_state_internal(build_safe_test_state())
    }

    fn preview_udev_rule(&self) -> Result<UdevRulePreview, AppError> {
        let snapshot = self.passive_snapshot();
        Ok(snapshot
            .candidate
            .map(|device| build_udev_rule_preview(device.vendor_id, device.product_id))
            .unwrap_or_else(unavailable_udev_rule_preview))
    }

    fn detected_hid_ids(&self) -> Result<Option<(u16, u16)>, AppError> {
        let snapshot = self.passive_snapshot();
        Ok(snapshot
            .candidate
            .map(|device| (device.vendor_id, device.product_id)))
    }

    /// Manual-only access probe. This is the ONLY diagnostics path that opens
    /// a device, it runs solely on an explicit user click, it refuses keyboard
    /// interfaces, and it drops the handle immediately. It never writes.
    fn probe_hid_access(&self) -> Result<HidAccessProbe, AppError> {
        self.ensure_active_access_allowed()?;

        let snapshot = self.passive_snapshot();
        let device = snapshot.candidate.clone().ok_or(AppError::DeviceNotFound)?;
        let summaries = self.interface_summaries(&snapshot);
        let running_as_root = is_running_as_root();

        let target = match select_probe_target(&summaries) {
            Ok(target) => target,
            Err(reason) => {
                return Ok(blocked_probe(&device, reason));
            }
        };
        let path = match target.path.clone() {
            Some(path) => path,
            None => {
                return Ok(blocked_probe(
                    &device,
                    "The eligible interface has no usable device path.".to_string(),
                ));
            }
        };

        // Open the single safe interface and drop the handle right away. For
        // a LampArray interface, additionally read its read-only attributes
        // report (lamp count etc.) before closing — still no writes.
        let (can_open, open_error, lamp_array_attributes) = match self.opener.open_path(&path) {
            Ok(handle) => {
                let attributes = if target.is_lamp_array {
                    self.read_lamp_array_attributes(&snapshot, &path, handle.as_ref())
                } else {
                    None
                };
                drop(handle);
                (true, None, attributes)
            }
            Err(error) => (false, Some(error.to_user_message()), None),
        };

        Ok(build_hid_access_probe(
            &device,
            can_open,
            open_error.as_deref(),
            running_as_root,
            lamp_array_attributes,
        ))
    }

    fn diagnostics(&self) -> Result<DiagnosticsReport, AppError> {
        let snapshot = self.passive_snapshot();
        let detected_device = self.device_info_from_snapshot(&snapshot);
        let hid_devices: Vec<HidDeviceSummary> = snapshot
            .devices
            .iter()
            .map(|device| device.to_summary(false))
            .collect();
        let hid_interfaces = self.interface_summaries(&snapshot);
        let eligible_rgb_interface_count = hid_interfaces
            .iter()
            .filter(|interface| interface.eligible_for_rgb_probe)
            .count();
        let known_supported_lenovo_rgb_device = snapshot
            .candidate
            .as_ref()
            .map(|device| device.to_summary(true));
        let supported_detected = known_supported_lenovo_rgb_device.is_some();
        let running_as_root = is_running_as_root();
        let effective_writes_enabled = self.effective_writes_enabled(&snapshot);
        let experimental_override_active = self.experimental_override_active(&snapshot);
        let write_allowlist_source = self.write_allowlist_source(&snapshot);
        let supported_effects = supported_lighting_effects(&detected_device.capabilities);
        let unsupported_effects = all_lighting_effects()
            .into_iter()
            .filter(|effect| !supported_effects.contains(effect))
            .collect::<Vec<_>>();
        let last_payload_bytes = self.last_payload();
        let last_payload_hex = last_payload_bytes
            .as_ref()
            .map(|payload| format_bytes(payload));
        let payload_preview = last_payload_bytes.as_ref().and_then(|payload| {
            (payload.len() == FEATURE_REPORT_LEN).then(|| decode_payload_preview(payload))
        });

        let mut notes = vec![
            "Diagnostics are passive: they read DMI and hidraw sysfs metadata only and never open HID devices.".to_string(),
            "The backend uses hidapi with the Linux hidraw backend. The libusb backend is forbidden because it can detach the kernel driver from the internal keyboard.".to_string(),
            "HID access probing is manual-only: use the \"Probe HID access\" button. It briefly opens only the single eligible RGB-control interface (vendor-defined or LampArray) and, for a LampArray, reads its read-only attributes.".to_string(),
            "Only known ITE 0x048d Lenovo Legion/LOQ 4-zone RGB devices are enabled for writes.".to_string(),
        ];

        if self.hid_access_disabled {
            notes.push(
                "HID access disabled by safety flag (LEGIONGLOW_DISABLE_HID=1). Detection is DMI/sysfs-only and all active HID operations are refused."
                    .to_string(),
            );
        }

        if self.dry_run_enabled() {
            notes.push(
                "Dry-run mode is active: detection and payload generation are real, but feature reports are not sent."
                    .to_string(),
            );
        }

        if self.dry_run_enabled()
            && snapshot
                .candidate
                .as_ref()
                .map(|device| device.dry_run_protocol_candidate && !device.supported_for_writes)
                .unwrap_or(false)
        {
            notes.push(
                "This product ID is enabled only for dry-run payload generation. Real writes remain blocked."
                    .to_string(),
            );
        }

        let mut warnings = Vec::new();

        if effective_writes_enabled {
            warnings
                .push("Lenovo HID backend active. Real hardware writes are enabled.".to_string());
        } else if self.writes_enabled() {
            warnings.push(
                "Real write mode was requested, but writes are blocked because the detected product or interface layout is not enabled for writes."
                    .to_string(),
            );
        }

        if experimental_override_active {
            warnings.push(
                "EXPERIMENTAL product ID override is active for the detected device. This product is not in the built-in safe write list; real HID feature reports may be sent. Validate with dry-run and the safe test payload first."
                    .to_string(),
            );
        }

        warnings.extend(self.experimental_allow.warnings.clone());

        if !self.hid_access_disabled && snapshot.devices.is_empty() {
            warnings
                .push("No ITE 0x048d HID devices were found in hidraw sysfs metadata.".to_string());
        }

        for device in &snapshot.devices {
            if !device.known {
                warnings.push(format!(
                    "ITE HID device {:04x}:{:04x} found, but product ID is not in the supported write list.",
                    device.vendor_id, device.product_id
                ));
            } else if !device.supported_for_writes {
                warnings.push(format!(
                    "Recognized Lenovo HID device {:04x}:{:04x}, but it is not enabled for writes yet.",
                    device.vendor_id, device.product_id
                ));
            }
        }

        if supported_detected && eligible_rgb_interface_count == 0 && !self.hid_access_disabled {
            warnings.push(
                "No safe RGB-control HID interface was identified. LegionGlow will not open this device."
                    .to_string(),
            );
        }

        let capabilities = detected_device.capabilities.clone();

        Ok(DiagnosticsReport {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            backend_mode: self.backend_name().to_string(),
            dmi_sys_vendor: snapshot.dmi.sys_vendor.clone(),
            dmi_product_name: snapshot.dmi.product_name.clone(),
            dmi_product_version: snapshot.dmi.product_version.clone(),
            detected_device,
            hid_devices,
            hid_interfaces,
            eligible_rgb_interface_count,
            hid_access_disabled_by_safety_flag: self.hid_access_disabled,
            known_supported_lenovo_rgb_device,
            // Passive diagnostics never hold a HID handle open.
            hid_device_opened: false,
            // Probing is manual-only; diagnostics never attempts an open.
            hid_access_probe: None,
            supported_effects,
            unsupported_effects,
            capabilities,
            real_hardware_backend_available: supported_detected,
            real_hardware_writes_enabled: effective_writes_enabled,
            dry_run_enabled: self.dry_run_enabled(),
            experimental_override_active,
            write_allowlist_source: write_allowlist_source.as_str().to_string(),
            requires_user_caution: self.mode.requires_user_caution(),
            // Without an open attempt there is no permission evidence.
            likely_permission_issue: false,
            running_as_root,
            last_payload_hex,
            payload_preview,
            notes,
            warnings,
        })
    }
}

/// Probe result for a device we refuse to open (no safe interface, ambiguous
/// metadata, or missing path). `can_open` is false and no open was attempted.
fn blocked_probe(device: &LenovoHidDeviceInfo, reason: String) -> HidAccessProbe {
    HidAccessProbe {
        vendor_id: format!("0x{:04x}", device.vendor_id),
        product_id: format!("0x{:04x}", device.product_id),
        label: device.label.clone(),
        manufacturer: device.manufacturer_string.clone(),
        product: device.product_string.clone(),
        path_available: !device.path.is_empty(),
        can_open: false,
        failure_kind: Some(HidOpenFailureKind::UnsupportedProduct),
        raw_error: None,
        user_message: reason,
        recommended_action:
            "LegionGlow only opens a single verified RGB-control interface (vendor-defined or \
             LampArray). Review the interface table in diagnostics; no action will open this \
             device."
                .to_string(),
        lamp_array_attributes: None,
    }
}

fn format_bytes(payload: &[u8]) -> String {
    payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_lenovo_state() -> KeyboardState {
    let mut state = KeyboardState::default_static();
    state.secondary_color = None;
    state
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{HidHandle, HidOpener, LenovoKeyboardDriver};
    use crate::{
        app::error::AppError,
        domain::HidOpenFailureKind,
        drivers::keyboard_driver::KeyboardDriver,
        infrastructure::{
            linux::hidraw::HidrawInterfaceMetadata,
            storage::settings_repository::{BackendMode, ExperimentalAllowlist},
        },
    };

    struct NoopHandle;

    impl HidHandle for NoopHandle {
        fn send_feature_report(&self, _payload: &[u8]) -> Result<(), AppError> {
            Ok(())
        }

        fn read_feature_report(&self, _report_id: u8) -> Result<Vec<u8>, AppError> {
            Err(AppError::HidError(
                "no feature reports in this fake".to_string(),
            ))
        }
    }

    /// Fake handle that answers GET_FEATURE with a canned LampArray
    /// attributes report (4 lamps, kind Keyboard) and counts reads.
    struct LampArrayHandle {
        feature_reads: Arc<AtomicUsize>,
    }

    impl HidHandle for LampArrayHandle {
        fn send_feature_report(&self, _payload: &[u8]) -> Result<(), AppError> {
            panic!("the probe must never SEND a feature report");
        }

        fn read_feature_report(&self, report_id: u8) -> Result<Vec<u8>, AppError> {
            self.feature_reads.fetch_add(1, Ordering::SeqCst);
            Ok(lamp_array_attributes_report(report_id))
        }
    }

    struct RecordingLampArrayHandle {
        feature_reads: Arc<AtomicUsize>,
        sent_reports: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl HidHandle for RecordingLampArrayHandle {
        fn send_feature_report(&self, payload: &[u8]) -> Result<(), AppError> {
            self.sent_reports.lock().unwrap().push(payload.to_vec());
            Ok(())
        }

        fn read_feature_report(&self, report_id: u8) -> Result<Vec<u8>, AppError> {
            self.feature_reads.fetch_add(1, Ordering::SeqCst);
            Ok(lamp_array_attributes_report(report_id))
        }
    }

    fn lamp_array_attributes_report(report_id: u8) -> Vec<u8> {
        let mut bytes = vec![report_id];
        bytes.extend_from_slice(&4u16.to_le_bytes()); // LampCount
        bytes.extend_from_slice(&[0u8; 12]); // bounding box
        bytes.extend_from_slice(&1u32.to_le_bytes()); // kind: Keyboard
        bytes.extend_from_slice(&33_333u32.to_le_bytes()); // min interval
        bytes
    }

    /// Counts every open attempt and records the last opened path.
    struct CountingOpener {
        opens: Arc<AtomicUsize>,
        last_path: Arc<Mutex<Option<String>>>,
    }

    impl HidOpener for CountingOpener {
        fn open_path(&self, path: &str) -> Result<Box<dyn HidHandle>, AppError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            *self.last_path.lock().unwrap() = Some(path.to_string());
            Ok(Box::new(NoopHandle))
        }
    }

    /// Opener that hands out LampArray-attribute-answering handles.
    struct LampArrayOpener {
        opens: Arc<AtomicUsize>,
        feature_reads: Arc<AtomicUsize>,
        last_path: Arc<Mutex<Option<String>>>,
    }

    impl HidOpener for LampArrayOpener {
        fn open_path(&self, path: &str) -> Result<Box<dyn HidHandle>, AppError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            *self.last_path.lock().unwrap() = Some(path.to_string());
            Ok(Box::new(LampArrayHandle {
                feature_reads: self.feature_reads.clone(),
            }))
        }
    }

    struct RecordingLampArrayOpener {
        opens: Arc<AtomicUsize>,
        feature_reads: Arc<AtomicUsize>,
        last_path: Arc<Mutex<Option<String>>>,
        sent_reports: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl HidOpener for RecordingLampArrayOpener {
        fn open_path(&self, path: &str) -> Result<Box<dyn HidHandle>, AppError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            *self.last_path.lock().unwrap() = Some(path.to_string());
            Ok(Box::new(RecordingLampArrayHandle {
                feature_reads: self.feature_reads.clone(),
                sent_reports: self.sent_reports.clone(),
            }))
        }
    }

    struct PanickingOpener;

    impl HidOpener for PanickingOpener {
        fn open_path(&self, _path: &str) -> Result<Box<dyn HidHandle>, AppError> {
            panic!("HID open must never be reached by this code path");
        }
    }

    fn interface_with_pairs(
        product_id: u16,
        dev_path: &str,
        interface_number: i32,
        usage_page: Option<u16>,
        usage: Option<u16>,
        all_usage_pairs: Vec<(u16, u16)>,
    ) -> HidrawInterfaceMetadata {
        HidrawInterfaceMetadata {
            dev_path: dev_path.to_string(),
            vendor_id: 0x048d,
            product_id,
            hid_name: Some("ITE Tech. Inc. ITE Device".to_string()),
            interface_number: Some(interface_number),
            usage_page,
            usage,
            all_usage_pairs,
            manufacturer: Some("ITE Tech. Inc.".to_string()),
            product: Some("ITE Device".to_string()),
            lamp_array_reports: crate::infrastructure::linux::hidraw::LampArrayReportIds::default(),
        }
    }

    fn interface(
        product_id: u16,
        dev_path: &str,
        interface_number: i32,
        usage_page: Option<u16>,
        usage: Option<u16>,
    ) -> HidrawInterfaceMetadata {
        let pairs = match (usage_page, usage) {
            (Some(page), Some(usage)) => vec![(page, usage)],
            _ => Vec::new(),
        };
        interface_with_pairs(
            product_id,
            dev_path,
            interface_number,
            usage_page,
            usage,
            pairs,
        )
    }

    /// A vendor-protocol layout: keyboard input, consumer control, and one
    /// vendor-defined RGB interface, all on the same VID/PID.
    fn vendor_protocol_interfaces() -> Vec<HidrawInterfaceMetadata> {
        vec![
            interface(0xc995, "/dev/hidraw0", 0, Some(0x01), Some(0x06)),
            interface(0xc995, "/dev/hidraw1", 1, Some(0x0c), Some(0x01)),
            interface(0xc995, "/dev/hidraw2", 2, Some(0xff89), Some(0x01)),
        ]
    }

    fn c693_vendor_defined_interfaces() -> Vec<HidrawInterfaceMetadata> {
        vec![
            interface(0xc693, "/dev/hidraw0", 0, Some(0x01), Some(0x06)),
            interface(0xc693, "/dev/hidraw1", 1, Some(0x0c), Some(0x01)),
            interface(0xc693, "/dev/hidraw2", 2, Some(0xff89), Some(0x01)),
        ]
    }

    fn driver_with(
        mode: BackendMode,
        allow: ExperimentalAllowlist,
        hid_disabled: bool,
        opener: Box<dyn HidOpener>,
        interfaces: Vec<HidrawInterfaceMetadata>,
    ) -> LenovoKeyboardDriver {
        LenovoKeyboardDriver::with_dependencies(
            mode,
            allow,
            hid_disabled,
            opener,
            Box::new(move || interfaces.clone()),
        )
    }

    fn counting_driver(
        mode: BackendMode,
        interfaces: Vec<HidrawInterfaceMetadata>,
    ) -> (
        LenovoKeyboardDriver,
        Arc<AtomicUsize>,
        Arc<Mutex<Option<String>>>,
    ) {
        let opens = Arc::new(AtomicUsize::new(0));
        let last_path = Arc::new(Mutex::new(None));
        let opener = CountingOpener {
            opens: opens.clone(),
            last_path: last_path.clone(),
        };
        let driver = driver_with(
            mode,
            ExperimentalAllowlist::default(),
            false,
            Box::new(opener),
            interfaces,
        );
        (driver, opens, last_path)
    }

    #[test]
    fn constructor_never_enumerates_or_opens_hid() {
        // Both the opener and the interface source panic if touched: building
        // the driver (what app bootstrap does) must run no HID code at all.
        let _driver = LenovoKeyboardDriver::with_dependencies(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            Box::new(|| panic!("interface enumeration must not run during startup")),
        );
    }

    #[test]
    fn passive_detection_and_diagnostics_never_open_hid() {
        let (driver, opens, _) =
            counting_driver(BackendMode::LenovoHidDryRun, vendor_protocol_interfaces());

        driver.detect_device().expect("detect");
        driver.diagnostics().expect("diagnostics");
        driver.preview_udev_rule().expect("udev preview");
        driver.detected_hid_ids().expect("ids");

        assert_eq!(opens.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn dry_run_set_state_never_opens_hid() {
        let (driver, opens, _) =
            counting_driver(BackendMode::LenovoHidDryRun, vendor_protocol_interfaces());

        driver
            .set_state(crate::domain::KeyboardState::default_static())
            .expect("dry-run set_state");

        assert_eq!(opens.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn diagnostics_report_marks_probe_as_manual_only() {
        let (driver, _, _) =
            counting_driver(BackendMode::LenovoHidDryRun, vendor_protocol_interfaces());

        let report = driver.diagnostics().expect("diagnostics");

        assert!(report.hid_access_probe.is_none());
        assert!(!report.hid_device_opened);
        assert_eq!(report.eligible_rgb_interface_count, 1);
        assert_eq!(report.hid_interfaces.len(), 3);
    }

    #[test]
    fn explicit_probe_opens_only_the_vendor_defined_interface_once() {
        let (driver, opens, last_path) =
            counting_driver(BackendMode::LenovoHidDryRun, vendor_protocol_interfaces());

        let probe = driver.probe_hid_access().expect("probe");

        assert!(probe.can_open);
        assert_eq!(opens.load(Ordering::SeqCst), 1);
        assert_eq!(
            last_path.lock().unwrap().as_deref(),
            Some("/dev/hidraw2"),
            "probe must target the vendor-defined interface, never the keyboard"
        );
    }

    #[test]
    fn probe_refuses_when_only_keyboard_interfaces_exist() {
        let interfaces = vec![
            interface(0xc693, "/dev/hidraw0", 0, Some(0x01), Some(0x06)),
            interface(0xc693, "/dev/hidraw1", 1, Some(0x07), Some(0x00)),
        ];
        let driver = driver_with(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            interfaces,
        );

        let probe = driver.probe_hid_access().expect("probe result");

        assert!(!probe.can_open);
        assert_eq!(
            probe.failure_kind,
            Some(HidOpenFailureKind::UnsupportedProduct)
        );
        assert!(probe
            .user_message
            .contains("No safe RGB-control HID interface was identified"));
    }

    #[test]
    fn probe_refuses_when_usage_metadata_is_missing() {
        let interfaces = vec![interface(0xc693, "/dev/hidraw0", 0, None, None)];
        let driver = driver_with(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            interfaces,
        );

        let probe = driver.probe_hid_access().expect("probe result");

        assert!(!probe.can_open);
    }

    #[test]
    fn probe_refuses_when_multiple_vendor_interfaces_are_ambiguous() {
        let interfaces = vec![
            interface(0xc693, "/dev/hidraw0", 0, Some(0xff89), Some(0x01)),
            interface(0xc693, "/dev/hidraw1", 1, Some(0xff89), Some(0x02)),
        ];
        let driver = driver_with(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            interfaces,
        );

        let probe = driver.probe_hid_access().expect("probe result");

        assert!(!probe.can_open);
        assert!(probe.user_message.contains("blocked"));
    }

    /// Mirrors the actual Lenovo LOQ 17IRX10 hardware: interface 0 is a
    /// composite report descriptor (vendor-defined collections AND the
    /// Generic Desktop Keyboard collection that backs the laptop keyboard),
    /// interface 1 is a HID LampArray with an attributes feature report.
    fn real_c693_interfaces() -> Vec<HidrawInterfaceMetadata> {
        let keyboard = interface_with_pairs(
            0xc693,
            "/dev/hidraw0",
            0,
            Some(0xff89),
            Some(0x10),
            vec![
                (0xff89, 0x0010),
                (0xff89, 0x0007),
                (0xff89, 0x00cc),
                (0x0001, 0x0006),
                (0xff99, 0x0010),
                (0x000c, 0x0001),
                (0x0001, 0x000c),
            ],
        );
        let mut lamp_array = interface_with_pairs(
            0xc693,
            "/dev/hidraw1",
            1,
            Some(0x59),
            Some(0x01),
            vec![(0x0059, 0x0001)],
        );
        lamp_array.lamp_array_reports.attributes = Some(0x01);
        lamp_array.lamp_array_reports.multi_update = Some(0x04);
        lamp_array.lamp_array_reports.range_update = Some(0x05);
        lamp_array.lamp_array_reports.control = Some(0x06);
        // (range_update is what the LampArray write path uses)
        vec![keyboard, lamp_array]
    }

    #[test]
    fn real_c693_layout_probes_only_the_lamp_array_interface() {
        // Interface 1 (LampArray) is the single eligible interface; the probe
        // must open it, read only its read-only attributes report, and never
        // touch interface 0 (the keyboard).
        let opens = Arc::new(AtomicUsize::new(0));
        let feature_reads = Arc::new(AtomicUsize::new(0));
        let last_path = Arc::new(Mutex::new(None));
        let driver = driver_with(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            false,
            Box::new(LampArrayOpener {
                opens: opens.clone(),
                feature_reads: feature_reads.clone(),
                last_path: last_path.clone(),
            }),
            real_c693_interfaces(),
        );

        let report = driver.diagnostics().expect("diagnostics");
        assert_eq!(report.eligible_rgb_interface_count, 1);
        assert!(report.hid_interfaces[0].is_keyboard_input);
        assert!(report.hid_interfaces[1].is_lamp_array);
        // Diagnostics stay passive even with an eligible interface present.
        assert_eq!(opens.load(Ordering::SeqCst), 0);

        let probe = driver.probe_hid_access().expect("probe");
        assert!(probe.can_open);
        assert_eq!(opens.load(Ordering::SeqCst), 1);
        assert_eq!(last_path.lock().unwrap().as_deref(), Some("/dev/hidraw1"));
        assert_eq!(feature_reads.load(Ordering::SeqCst), 1);

        let attributes = probe.lamp_array_attributes.expect("attributes");
        assert_eq!(attributes.lamp_count, 4);
        assert_eq!(attributes.kind_label, "Keyboard");
    }

    #[test]
    fn real_c693_layout_uses_lamp_array_protocol() {
        // c693: interface 0 is composite (keyboard+vendor), interface 1 is
        // LampArray. Writes must use the standard LampArray interface and
        // never send legacy vendor bytes through the keyboard interface.
        let opens = Arc::new(AtomicUsize::new(0));
        let feature_reads = Arc::new(AtomicUsize::new(0));
        let last_path = Arc::new(Mutex::new(None));
        let sent_reports = Arc::new(Mutex::new(Vec::new()));
        let driver = driver_with(
            BackendMode::LenovoHid,
            ExperimentalAllowlist::default(),
            false,
            Box::new(RecordingLampArrayOpener {
                opens: opens.clone(),
                feature_reads: feature_reads.clone(),
                last_path: last_path.clone(),
                sent_reports: sent_reports.clone(),
            }),
            real_c693_interfaces(),
        );

        driver
            .send_safe_test_payload()
            .expect("safe LampArray write");

        // Writes go to interface 1 (LampArray), not interface 0
        // (composite keyboard+vendor).
        assert_eq!(opens.load(Ordering::SeqCst), 1);
        assert_eq!(feature_reads.load(Ordering::SeqCst), 1);
        assert_eq!(last_path.lock().unwrap().as_deref(), Some("/dev/hidraw1"));
        let sent_reports = sent_reports.lock().unwrap();
        // LampArray protocol: control report, then one range-update per zone.
        // The mock attributes report declares 4 lamps, so each of the 4 zones
        // maps to a single lamp. The safe-test payload is dim blue (0,0,32) at
        // hardware intensity 2 (brightness 1 → 1 * 255 / 100).
        assert_eq!(sent_reports.len(), 5);
        assert_eq!(sent_reports[0], vec![0x06, 0x00]);
        assert_eq!(sent_reports[1], vec![0x05, 0x00, 0, 0, 0, 0, 0, 0, 32, 2]);
        assert_eq!(sent_reports[2], vec![0x05, 0x00, 1, 0, 1, 0, 0, 0, 32, 2]);
        assert_eq!(sent_reports[3], vec![0x05, 0x00, 2, 0, 2, 0, 0, 0, 32, 2]);
        assert_eq!(sent_reports[4], vec![0x05, 0x01, 3, 0, 3, 0, 0, 0, 32, 2]);

        let report = driver.diagnostics().expect("diagnostics");
        assert!(report.real_hardware_writes_enabled);
        assert_eq!(report.write_allowlist_source, "built-in");
        assert!(report.last_payload_hex.is_some());
        assert!(report.payload_preview.is_none());
        assert_eq!(
            report
                .supported_effects
                .iter()
                .map(|effect| format!("{effect:?}"))
                .collect::<Vec<_>>(),
            vec!["Static".to_string(), "Off".to_string()]
        );
    }

    #[test]
    fn c693_without_lamp_array_interface_is_not_writable() {
        // If c693 only exposes the composite keyboard/vendor interface, the
        // LampArray write protocol has nowhere safe to send.
        let keyboard_only = real_c693_interfaces().into_iter().take(1).collect();
        let driver = driver_with(
            BackendMode::LenovoHid,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            keyboard_only,
        );

        let report = driver.diagnostics().expect("diagnostics");
        assert!(!report.real_hardware_writes_enabled);

        let error = driver
            .send_safe_test_payload()
            .expect_err("no LampArray interface must block writes");
        assert!(error
            .to_user_message()
            .contains("No safe RGB-control HID interface"));
    }

    #[test]
    fn ite_vendor_protocol_refuses_when_only_lamp_array_interface_present() {
        // ITE vendor write requires a vendor-defined interface. A pure LampArray
        // interface (page 0x59, not vendor-defined) must not receive vendor bytes.
        let mut lamp_array = interface_with_pairs(
            0xc965,
            "/dev/hidraw1",
            1,
            Some(0x59),
            Some(0x01),
            vec![(0x0059, 0x0001)],
        );
        lamp_array.lamp_array_reports.attributes = Some(0x01);
        let driver = driver_with(
            BackendMode::LenovoHid,
            ExperimentalAllowlist::default(),
            false,
            Box::new(PanickingOpener),
            vec![lamp_array],
        );

        let error = driver
            .set_state(crate::domain::KeyboardState::default_static())
            .expect_err("must fail without a vendor-defined interface");
        assert!(error.to_user_message().contains("vendor-defined interface"));
    }

    #[test]
    fn safety_flag_blocks_probe_and_writes() {
        let driver = driver_with(
            BackendMode::LenovoHid,
            ExperimentalAllowlist::default(),
            true,
            Box::new(PanickingOpener),
            vendor_protocol_interfaces(),
        );

        let probe_error = driver.probe_hid_access().expect_err("probe must refuse");
        assert!(probe_error
            .to_user_message()
            .contains("LEGIONGLOW_DISABLE_HID"));

        let write_error = driver
            .send_safe_test_payload()
            .expect_err("write must refuse");
        assert!(write_error
            .to_user_message()
            .contains("LEGIONGLOW_DISABLE_HID"));
    }

    #[test]
    fn safety_flag_keeps_diagnostics_working_without_hid() {
        let driver = LenovoKeyboardDriver::with_dependencies(
            BackendMode::LenovoHidDryRun,
            ExperimentalAllowlist::default(),
            true,
            Box::new(PanickingOpener),
            Box::new(|| panic!("hidraw enumeration must not run when the safety flag is set")),
        );

        let report = driver.diagnostics().expect("diagnostics");

        assert!(report.hid_access_disabled_by_safety_flag);
        assert!(report.hid_interfaces.is_empty());
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("LEGIONGLOW_DISABLE_HID")));
    }

    #[test]
    fn c693_refuses_dedicated_vendor_interface_without_lamp_array() {
        // c693 is bound to the LampArray write protocol. Even if a test layout
        // exposes a dedicated vendor interface, it must not receive legacy ITE
        // vendor payloads for this product.
        let (driver, opens, last_path) =
            counting_driver(BackendMode::LenovoHid, c693_vendor_defined_interfaces());

        let error = driver
            .send_safe_test_payload()
            .expect_err("c693 must refuse vendor-only layouts");

        assert_eq!(opens.load(Ordering::SeqCst), 0);
        assert_eq!(last_path.lock().unwrap().as_deref(), None);
        assert!(error
            .to_user_message()
            .contains("configured for HID LampArray writes"));

        let report = driver.diagnostics().expect("diagnostics");
        assert!(!report.real_hardware_writes_enabled);
        assert_eq!(report.write_allowlist_source, "built-in");
    }

    #[test]
    fn vid_pid_match_alone_never_enables_probing() {
        // Device is a known candidate by VID/PID, but no usage metadata is
        // available for any interface: nothing may be opened.
        let interfaces = vec![interface(0xc693, "/dev/hidraw0", 0, None, None)];
        let (driver, opens, _) = {
            let opens = Arc::new(AtomicUsize::new(0));
            let last_path = Arc::new(Mutex::new(None));
            let opener = CountingOpener {
                opens: opens.clone(),
                last_path: last_path.clone(),
            };
            let driver = driver_with(
                BackendMode::LenovoHidDryRun,
                ExperimentalAllowlist::default(),
                false,
                Box::new(opener),
                interfaces,
            );
            (driver, opens, last_path)
        };

        let probe = driver.probe_hid_access().expect("probe result");

        assert!(!probe.can_open);
        assert_eq!(opens.load(Ordering::SeqCst), 0);
    }
}
