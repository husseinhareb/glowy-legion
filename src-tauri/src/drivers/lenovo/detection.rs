use crate::{
    app::error::AppError,
    domain::{DeviceCapabilities, DeviceFamily, DeviceInfo, HidDeviceSummary},
    infrastructure::linux::{
        dmi::{read_dmi_info, DmiInfo},
        hidraw::HidrawInterfaceMetadata,
    },
};

pub const ITE_VENDOR_ID: u16 = 0x048d;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LenovoWriteProtocol {
    IteVendor,
    /// Standard HID LampArray (usage page 0x59) write path.
    HidLampArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownLenovoRgbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub label: &'static str,
    pub family: DeviceFamily,
    pub write_protocol: Option<LenovoWriteProtocol>,
    pub dry_run_protocol_candidate: bool,
}

pub const KNOWN_LENOVO_RGB_DEVICES: &[KnownLenovoRgbDevice] = &[
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc995,
        label: "Lenovo Legion 2024 Pro 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc994,
        label: "Lenovo Legion 2024 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc993,
        label: "Lenovo LOQ 2024 4-zone RGB",
        family: DeviceFamily::LenovoLoq,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc985,
        label: "Lenovo Legion 2023 Pro 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc984,
        label: "Lenovo Legion 2023 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc983,
        label: "Lenovo LOQ 2023 4-zone RGB",
        family: DeviceFamily::LenovoLoq,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc975,
        label: "Lenovo Legion 2022 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc973,
        label: "Lenovo IdeaPad Gaming 2022 4-zone RGB",
        family: DeviceFamily::LenovoUnknown,
        write_protocol: None,
        dry_run_protocol_candidate: false,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc965,
        label: "Lenovo Legion 2021 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc963,
        label: "Lenovo IdeaPad Gaming 2021 4-zone RGB",
        family: DeviceFamily::LenovoUnknown,
        write_protocol: None,
        dry_run_protocol_candidate: false,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc955,
        label: "Lenovo Legion 2020 4-zone RGB",
        family: DeviceFamily::LenovoLegion,
        write_protocol: Some(LenovoWriteProtocol::IteVendor),
        dry_run_protocol_candidate: true,
    },
    KnownLenovoRgbDevice {
        vendor_id: ITE_VENDOR_ID,
        product_id: 0xc693,
        label: "Lenovo LOQ 17IRX10 4-zone RGB",
        family: DeviceFamily::LenovoLoq,
        // The real LOQ 17IRX10 exposes a separate HID Lighting & Illumination
        // LampArray interface. Do not send legacy ITE vendor bytes through the
        // composite keyboard/vendor interface for this product.
        write_protocol: Some(LenovoWriteProtocol::HidLampArray),
        dry_run_protocol_candidate: true,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LenovoHidDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    /// Representative hidraw path of one interface. Informational only — the
    /// device is never opened through this during detection.
    pub path: String,
    pub manufacturer_string: Option<String>,
    pub product_string: Option<String>,
    pub label: String,
    pub known: bool,
    pub supported_for_writes: bool,
    pub write_protocol: Option<LenovoWriteProtocol>,
    pub dry_run_protocol_candidate: bool,
    pub family: DeviceFamily,
    pub capabilities: DeviceCapabilities,
    pub dmi: DmiInfo,
}

impl LenovoHidDeviceInfo {
    pub fn device_id(&self) -> String {
        format!("hid-{:04x}-{:04x}", self.vendor_id, self.product_id)
    }

    pub fn product_name(&self) -> String {
        self.dmi
            .product_name
            .clone()
            .or_else(|| self.product_string.clone())
            .unwrap_or_else(|| self.label.clone())
    }

    pub fn to_device_info(&self, backend: &str, supported: bool) -> DeviceInfo {
        DeviceInfo {
            id: self.device_id(),
            vendor: self
                .dmi
                .sys_vendor
                .clone()
                .or_else(|| self.manufacturer_string.clone())
                .unwrap_or_else(|| "Lenovo / ITE".to_string()),
            product_name: self.product_name(),
            family: self.family.clone(),
            supported,
            backend: backend.to_string(),
            capabilities: if supported {
                self.capabilities.clone()
            } else {
                DeviceCapabilities::unsupported()
            },
        }
    }

    pub fn to_summary(&self, include_path: bool) -> HidDeviceSummary {
        HidDeviceSummary {
            vendor_id: format!("0x{:04x}", self.vendor_id),
            product_id: format!("0x{:04x}", self.product_id),
            manufacturer: self.manufacturer_string.clone(),
            product: self
                .product_string
                .clone()
                .or_else(|| Some(self.label.clone())),
            path: include_path.then(|| self.path.clone()),
            known: self.known,
            supported_for_writes: self.supported_for_writes,
        }
    }
}

/// Pick the active RGB candidate from passively enumerated interface metadata.
/// Pure and passive: nothing here opens, claims, or probes any device.
pub fn detect_lenovo_rgb_device_from_metadata(
    interfaces: &[HidrawInterfaceMetadata],
    dmi: &DmiInfo,
    allow_dry_run_candidates: bool,
) -> Result<LenovoHidDeviceInfo, AppError> {
    list_possible_lenovo_hid_devices_from_metadata(interfaces, dmi)
        .into_iter()
        .find(|device| {
            is_known_supported_device(device.vendor_id, device.product_id)
                || (allow_dry_run_candidates && device.dry_run_protocol_candidate)
        })
        .ok_or(AppError::DeviceNotFound)
}

/// List unique ITE devices (one entry per VID/PID) from passive hidraw
/// metadata. Multiple interfaces of one USB device collapse into one entry.
pub fn list_possible_lenovo_hid_devices_from_metadata(
    interfaces: &[HidrawInterfaceMetadata],
    dmi: &DmiInfo,
) -> Vec<LenovoHidDeviceInfo> {
    let mut devices: Vec<LenovoHidDeviceInfo> = Vec::new();

    for interface in interfaces {
        if interface.vendor_id != ITE_VENDOR_ID {
            continue;
        }
        if devices.iter().any(|existing| {
            existing.vendor_id == interface.vendor_id && existing.product_id == interface.product_id
        }) {
            continue;
        }

        let known = known_device(interface.vendor_id, interface.product_id);
        let write_protocol = known.and_then(|device| device.write_protocol);
        let supported_for_writes = write_protocol.is_some();
        let dry_run_protocol_candidate = known
            .map(|device| device.dry_run_protocol_candidate)
            .unwrap_or(false);
        let capabilities = match write_protocol {
            Some(LenovoWriteProtocol::HidLampArray) => DeviceCapabilities::lenovo_lamp_array_rgb(),
            Some(LenovoWriteProtocol::IteVendor) => DeviceCapabilities::lenovo_4_zone_rgb(),
            None if dry_run_protocol_candidate => DeviceCapabilities::lenovo_4_zone_rgb(),
            None => DeviceCapabilities::unsupported(),
        };

        devices.push(LenovoHidDeviceInfo {
            vendor_id: interface.vendor_id,
            product_id: interface.product_id,
            path: interface.dev_path.clone(),
            manufacturer_string: interface.manufacturer.clone(),
            product_string: interface
                .product
                .clone()
                .or_else(|| interface.hid_name.clone()),
            label: known
                .map(|device| device.label.to_string())
                .unwrap_or_else(|| "Unknown ITE HID device".to_string()),
            known: known.is_some(),
            supported_for_writes,
            write_protocol,
            dry_run_protocol_candidate,
            family: known
                .map(|device| device.family)
                .unwrap_or(DeviceFamily::Unsupported),
            capabilities,
            dmi: dmi.clone(),
        });
    }

    devices
}

pub fn known_device(vendor_id: u16, product_id: u16) -> Option<&'static KnownLenovoRgbDevice> {
    KNOWN_LENOVO_RGB_DEVICES
        .iter()
        .find(|device| device.vendor_id == vendor_id && device.product_id == product_id)
}

#[allow(dead_code)]
pub fn classify_lenovo_model(product_name: &str) -> DeviceFamily {
    let normalized = product_name.to_ascii_lowercase();

    if normalized.contains("legion") {
        DeviceFamily::LenovoLegion
    } else if normalized.contains("loq") {
        DeviceFamily::LenovoLoq
    } else if normalized.contains("lenovo") {
        DeviceFamily::LenovoUnknown
    } else {
        DeviceFamily::Unsupported
    }
}

#[allow(dead_code)]
pub fn detect_from_linux_dmi() -> Option<DeviceFamily> {
    Some(classify_dmi_info(&read_dmi_info())).filter(|family| *family != DeviceFamily::Unsupported)
}

pub fn classify_dmi_info(dmi: &DmiInfo) -> DeviceFamily {
    let combined = [
        dmi.sys_vendor.as_deref(),
        dmi.product_name.as_deref(),
        dmi.product_version.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");

    classify_lenovo_model(&combined)
}

pub fn unsupported_device_from_dmi(backend: &str) -> DeviceInfo {
    let dmi = read_dmi_info();
    let family = classify_dmi_info(&dmi);
    let product_name = match (&dmi.product_name, &dmi.product_version) {
        (Some(name), Some(version)) if name != version => format!("{version} ({name})"),
        (Some(name), _) => name.clone(),
        (_, Some(version)) => version.clone(),
        _ => "No supported Lenovo Legion/LOQ 4-zone RGB HID device detected".to_string(),
    };

    DeviceInfo {
        id: "unsupported-lenovo-hid-device".to_string(),
        vendor: dmi.sys_vendor.unwrap_or_else(|| "Unknown".to_string()),
        product_name,
        family,
        supported: false,
        backend: backend.to_string(),
        capabilities: DeviceCapabilities::unsupported(),
    }
}

fn is_known_supported_device(vendor_id: u16, product_id: u16) -> bool {
    known_device(vendor_id, product_id)
        .map(|device| device.write_protocol.is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        classify_dmi_info, classify_lenovo_model, detect_lenovo_rgb_device_from_metadata,
        known_device, list_possible_lenovo_hid_devices_from_metadata, ITE_VENDOR_ID,
    };
    use crate::domain::DeviceFamily;
    use crate::infrastructure::linux::{dmi::DmiInfo, hidraw::HidrawInterfaceMetadata};

    fn metadata(vendor_id: u16, product_id: u16, dev_path: &str) -> HidrawInterfaceMetadata {
        HidrawInterfaceMetadata {
            dev_path: dev_path.to_string(),
            vendor_id,
            product_id,
            hid_name: None,
            interface_number: None,
            usage_page: None,
            usage: None,
            all_usage_pairs: Vec::new(),
            manufacturer: None,
            product: None,
            lamp_array_reports: crate::infrastructure::linux::hidraw::LampArrayReportIds::default(),
        }
    }

    #[test]
    fn passive_listing_dedupes_interfaces_and_filters_non_ite_devices() {
        let interfaces = vec![
            metadata(ITE_VENDOR_ID, 0xc693, "/dev/hidraw0"),
            metadata(ITE_VENDOR_ID, 0xc693, "/dev/hidraw1"),
            metadata(0x046d, 0xc52b, "/dev/hidraw2"),
        ];

        let devices =
            list_possible_lenovo_hid_devices_from_metadata(&interfaces, &DmiInfo::default());

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].product_id, 0xc693);
    }

    #[test]
    fn passive_detection_finds_lamp_array_supported_c693_in_real_mode() {
        let interfaces = vec![metadata(ITE_VENDOR_ID, 0xc693, "/dev/hidraw0")];

        assert!(
            detect_lenovo_rgb_device_from_metadata(&interfaces, &DmiInfo::default(), true).is_ok()
        );
        assert!(
            detect_lenovo_rgb_device_from_metadata(&interfaces, &DmiInfo::default(), false).is_ok()
        );
    }

    #[test]
    fn classifies_lenovo_models() {
        assert_eq!(
            classify_lenovo_model("Lenovo Legion Pro 7"),
            DeviceFamily::LenovoLegion
        );
        assert_eq!(
            classify_lenovo_model("Lenovo LOQ 15"),
            DeviceFamily::LenovoLoq
        );
        assert_eq!(
            classify_lenovo_model("Lenovo Notebook"),
            DeviceFamily::LenovoUnknown
        );
    }

    #[test]
    fn knows_public_ite_product_ids() {
        assert!(known_device(ITE_VENDOR_ID, 0xc995).is_some());
        assert!(known_device(ITE_VENDOR_ID, 0xffff).is_none());
    }

    #[test]
    fn keeps_known_ideapad_ids_disabled_for_initial_backend() {
        assert!(known_device(ITE_VENDOR_ID, 0xc973)
            .unwrap()
            .write_protocol
            .is_none());
        assert!(known_device(ITE_VENDOR_ID, 0xc963)
            .unwrap()
            .write_protocol
            .is_none());
    }

    #[test]
    fn recognizes_loq_17irx10_lamp_array_backend() {
        let device = known_device(ITE_VENDOR_ID, 0xc693).unwrap();

        assert_eq!(device.family, DeviceFamily::LenovoLoq);
        assert_eq!(
            device.write_protocol,
            Some(super::LenovoWriteProtocol::HidLampArray)
        );
        assert!(device.dry_run_protocol_candidate);
    }

    #[test]
    fn classifies_dmi_product_version_for_machine_type_models() {
        let dmi = DmiInfo {
            sys_vendor: Some("LENOVO".to_string()),
            product_name: Some("83JH".to_string()),
            product_version: Some("LOQ 17IRX10".to_string()),
        };

        assert_eq!(classify_dmi_info(&dmi), DeviceFamily::LenovoLoq);
    }
}
