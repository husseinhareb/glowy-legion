//! Passive hidraw metadata enumeration.
//!
//! Everything in this module reads sysfs files only. It never opens
//! `/dev/hidraw*` device nodes, never initializes a HID library, never claims
//! a USB interface, and never detaches a kernel driver. It is safe to run on
//! app startup and during diagnostics.

use std::fs;
use std::path::{Path, PathBuf};

const HIDRAW_SYSFS_ROOT: &str = "/sys/class/hidraw";

/// HID Lighting & Illumination usage page (LampArray).
pub const USAGE_PAGE_LIGHTING: u16 = 0x59;
/// LampArray application collection usage on the lighting page.
pub const USAGE_LAMP_ARRAY: u16 = 0x01;

/// Feature report IDs of the standard HID LampArray reports, extracted from a
/// report descriptor without opening the device. `None` means the report was
/// not declared (or the interface is not a LampArray).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LampArrayReportIds {
    /// LampArrayAttributesReport (usage 0x02) — read-only device summary.
    pub attributes: Option<u8>,
    /// LampAttributesRequestReport (usage 0x20).
    pub attributes_request: Option<u8>,
    /// LampAttributesResponseReport (usage 0x22).
    pub attributes_response: Option<u8>,
    /// LampMultiUpdateReport (usage 0x50).
    pub multi_update: Option<u8>,
    /// LampRangeUpdateReport (usage 0x60).
    pub range_update: Option<u8>,
    /// LampArrayControlReport (usage 0x70).
    pub control: Option<u8>,
}

/// Metadata about one hidraw node, collected without opening the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidrawInterfaceMetadata {
    /// Device node path, e.g. `/dev/hidraw3`. Present even if unreadable.
    pub dev_path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    /// HID_NAME from the kernel uevent, e.g. "ITE Tech. Inc. ITE Device".
    pub hid_name: Option<String>,
    /// USB `bInterfaceNumber` of the parent interface, if resolvable.
    pub interface_number: Option<i32>,
    /// Usage page of the FIRST top-level collection (display only — safety
    /// decisions must use `all_usage_pairs`).
    pub usage_page: Option<u16>,
    /// Usage of the FIRST top-level collection (display only).
    pub usage: Option<u16>,
    /// `(usage_page, usage)` of EVERY top-level collection in the report
    /// descriptor. One interface can mix vendor-defined and keyboard
    /// collections; classification must see all of them.
    pub all_usage_pairs: Vec<(u16, u16)>,
    /// USB device manufacturer string from sysfs.
    pub manufacturer: Option<String>,
    /// USB device product string from sysfs.
    pub product: Option<String>,
    /// LampArray feature report IDs parsed from the report descriptor. All
    /// `None` for non-LampArray interfaces.
    pub lamp_array_reports: LampArrayReportIds,
}

/// Enumerate hidraw interfaces through sysfs only. Returns an empty list when
/// sysfs is unavailable (non-Linux, sandboxed tests, no HID devices).
pub fn enumerate_hidraw_interfaces() -> Vec<HidrawInterfaceMetadata> {
    let entries = match fs::read_dir(HIDRAW_SYSFS_ROOT) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut interfaces: Vec<HidrawInterfaceMetadata> = entries
        .flatten()
        .filter_map(|entry| read_hidraw_metadata(&entry.path()))
        .collect();
    interfaces.sort_by(|a, b| a.dev_path.cmp(&b.dev_path));
    interfaces
}

fn read_hidraw_metadata(class_path: &Path) -> Option<HidrawInterfaceMetadata> {
    let name = class_path.file_name()?.to_str()?.to_string();
    if !name.starts_with("hidraw") {
        return None;
    }

    // `/sys/class/hidraw/hidrawN/device` is the HID device directory.
    let hid_device_dir = class_path.join("device");
    let uevent = fs::read_to_string(hid_device_dir.join("uevent")).ok()?;
    let (vendor_id, product_id) = parse_hid_id(&uevent)?;
    let hid_name = parse_uevent_value(&uevent, "HID_NAME");

    let descriptor = fs::read(hid_device_dir.join("report_descriptor")).ok();
    let all_usage_pairs = descriptor
        .as_deref()
        .map(parse_report_descriptor_usages)
        .unwrap_or_default();
    let lamp_array_reports = descriptor
        .as_deref()
        .map(parse_lamp_array_report_ids)
        .unwrap_or_default();
    let (usage_page, usage) = all_usage_pairs
        .first()
        .map(|(page, usage)| (Some(*page), Some(*usage)))
        .unwrap_or((None, None));

    // For USB HID devices the HID device's parent is the USB interface and the
    // grandparent is the USB device. These reads are best-effort.
    let usb_interface_dir = hid_device_dir.join("..");
    let usb_device_dir = hid_device_dir.join("../..");
    let interface_number = read_trimmed(&usb_interface_dir.join("bInterfaceNumber"))
        .and_then(|value| i32::from_str_radix(&value, 16).ok());
    let manufacturer = read_trimmed(&usb_device_dir.join("manufacturer"));
    let product = read_trimmed(&usb_device_dir.join("product"));

    Some(HidrawInterfaceMetadata {
        dev_path: format!("/dev/{name}"),
        vendor_id,
        product_id,
        hid_name,
        interface_number,
        usage_page,
        usage,
        all_usage_pairs,
        manufacturer,
        product,
        lamp_array_reports,
    })
}

fn read_trimmed(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parse `HID_ID=0003:0000048D:0000C693` into `(vendor_id, product_id)`.
pub fn parse_hid_id(uevent: &str) -> Option<(u16, u16)> {
    let value = parse_uevent_value(uevent, "HID_ID")?;
    let mut parts = value.split(':');
    let _bus = parts.next()?;
    let vendor = u32::from_str_radix(parts.next()?, 16).ok()?;
    let product = u32::from_str_radix(parts.next()?, 16).ok()?;
    Some((vendor as u16, product as u16))
}

fn parse_uevent_value(uevent: &str, key: &str) -> Option<String> {
    uevent
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parse ALL top-level application collections from a HID report descriptor,
/// returning one `(usage_page, usage)` pair per collection.
///
/// This must not stop at the first collection: real hardware (e.g. the ITE
/// `048d:c693` interface 0) packs vendor-defined collections AND a Generic
/// Desktop Keyboard collection into the same interface. Safety classification
/// has to see every collection so a keyboard usage anywhere poisons the
/// interface.
pub fn parse_report_descriptor_usages(bytes: &[u8]) -> Vec<(u16, u16)> {
    let mut pairs = Vec::new();
    let mut usage_page: Option<u16> = None;
    let mut pending_usage: Option<u16> = None;
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        let prefix = bytes[index];

        // Long item (0xfe): skip its declared payload.
        if prefix == 0xfe {
            let data_len = bytes.get(index + 1).copied().unwrap_or(0) as usize;
            index += 3 + data_len;
            continue;
        }

        let size = match prefix & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        };
        let tag_and_type = prefix & 0xfc;
        let data = read_item_data(bytes, index + 1, size);

        match tag_and_type {
            // Global: Usage Page (persists across collections)
            0x04 => usage_page = data.map(|value| value as u16),
            // Local: Usage — remember the most recent one at the top level.
            0x08 => {
                if depth == 0 {
                    pending_usage = data.map(|value| value as u16);
                }
            }
            // Main: Collection
            0xa0 => {
                if depth == 0 {
                    if let (Some(page), Some(usage)) = (usage_page, pending_usage) {
                        pairs.push((page, usage));
                    }
                }
                pending_usage = None;
                depth += 1;
            }
            // Main: End Collection
            0xc0 => depth = depth.saturating_sub(1),
            _ => {}
        }

        index += 1 + size;
    }

    pairs
}

/// Extract the feature report IDs of the standard LampArray reports from a
/// report descriptor. Purely passive: operates on descriptor bytes that were
/// already read from sysfs.
///
/// LampArray descriptors declare one logical collection per report (usage
/// 0x02/0x20/0x22/0x50/0x60/0x70 on the lighting page), each with its own
/// Report ID followed by Feature items. A Feature item is attributed to the
/// innermost open collection.
pub fn parse_lamp_array_report_ids(bytes: &[u8]) -> LampArrayReportIds {
    let mut ids = LampArrayReportIds::default();
    let mut usage_page: Option<u16> = None;
    let mut report_id: Option<u8> = None;
    let mut pending_usage: Option<(u16, u16)> = None;
    // Usage of each open collection, innermost last.
    let mut collection_usages: Vec<(u16, u16)> = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        let prefix = bytes[index];

        if prefix == 0xfe {
            let data_len = bytes.get(index + 1).copied().unwrap_or(0) as usize;
            index += 3 + data_len;
            continue;
        }

        let size = match prefix & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        };
        let tag_and_type = prefix & 0xfc;
        let data = read_item_data(bytes, index + 1, size);

        match tag_and_type {
            // Global: Usage Page
            0x04 => usage_page = data.map(|value| value as u16),
            // Global: Report ID
            0x84 => report_id = data.map(|value| value as u8),
            // Local: Usage. The 4-byte form carries its own usage page in the
            // high word.
            0x08 => {
                pending_usage = data.and_then(|value| {
                    let page = if size == 4 {
                        Some((value >> 16) as u16)
                    } else {
                        usage_page
                    };
                    page.map(|page| (page, value as u16))
                });
            }
            // Main: Collection
            0xa0 => {
                collection_usages.push(pending_usage.unwrap_or((0, 0)));
                pending_usage = None;
            }
            // Main: End Collection
            0xc0 => {
                collection_usages.pop();
            }
            // Main: Feature
            0xb0 => {
                if let (Some(id), Some(&(page, usage))) = (report_id, collection_usages.last()) {
                    if page == USAGE_PAGE_LIGHTING {
                        let slot = match usage {
                            0x02 => Some(&mut ids.attributes),
                            0x20 => Some(&mut ids.attributes_request),
                            0x22 => Some(&mut ids.attributes_response),
                            0x50 => Some(&mut ids.multi_update),
                            0x60 => Some(&mut ids.range_update),
                            0x70 => Some(&mut ids.control),
                            _ => None,
                        };
                        if let Some(slot) = slot {
                            slot.get_or_insert(id);
                        }
                    }
                }
                pending_usage = None;
            }
            // Main: Input / Output also consume pending local usages.
            0x80 | 0x90 => pending_usage = None,
            _ => {}
        }

        index += 1 + size;
    }

    ids
}

fn read_item_data(bytes: &[u8], start: usize, size: usize) -> Option<u32> {
    if size == 0 {
        return Some(0);
    }
    let slice = bytes.get(start..start + size)?;
    let mut value = 0u32;
    for (shift, byte) in slice.iter().enumerate() {
        value |= (*byte as u32) << (8 * shift);
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_hid_id, parse_lamp_array_report_ids, parse_report_descriptor_usages,
        LampArrayReportIds,
    };

    #[test]
    fn parses_hid_id_from_uevent() {
        let uevent = "DRIVER=hid-generic\nHID_ID=0003:0000048D:0000C693\nHID_NAME=ITE Tech. Inc. ITE Device\n";
        assert_eq!(parse_hid_id(uevent), Some((0x048d, 0xc693)));
    }

    #[test]
    fn rejects_uevent_without_hid_id() {
        assert_eq!(parse_hid_id("DRIVER=hid-generic\n"), None);
    }

    #[test]
    fn parses_keyboard_report_descriptor_usage() {
        // Usage Page (Generic Desktop), Usage (Keyboard), Collection (Application)
        let descriptor = [0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0xc0];
        assert_eq!(
            parse_report_descriptor_usages(&descriptor),
            vec![(0x0001, 0x0006)]
        );
    }

    #[test]
    fn parses_consumer_control_report_descriptor_usage() {
        // Usage Page (Consumer), Usage (Consumer Control), Collection
        let descriptor = [0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0xc0];
        assert_eq!(
            parse_report_descriptor_usages(&descriptor),
            vec![(0x000c, 0x0001)]
        );
    }

    #[test]
    fn parses_vendor_defined_report_descriptor_usage() {
        // Usage Page (Vendor 0xff89, two-byte form), Usage (0x01), Collection
        let descriptor = [0x06, 0x89, 0xff, 0x09, 0x01, 0xa1, 0x01, 0xc0];
        assert_eq!(
            parse_report_descriptor_usages(&descriptor),
            vec![(0xff89, 0x0001)]
        );
    }

    #[test]
    fn empty_descriptor_yields_no_usage_metadata() {
        assert!(parse_report_descriptor_usages(&[]).is_empty());
    }

    #[test]
    fn nested_collections_do_not_produce_extra_pairs() {
        let descriptor = [
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x06, // Usage (Keyboard)
            0xa1, 0x01, // Collection (Application)
            0x09, 0x01, // Usage inside the collection
            0xa1, 0x02, // nested Collection (Logical)
            0xc0, // End Collection
            0xc0, // End Collection
        ];
        assert_eq!(
            parse_report_descriptor_usages(&descriptor),
            vec![(0x0001, 0x0006)]
        );
    }

    #[test]
    fn composite_interface_reports_every_top_level_collection() {
        // Mirrors the real ITE 048d:c693 interface 0: vendor-defined
        // collections followed by a Generic Desktop Keyboard collection and a
        // Consumer Control collection, all in one report descriptor.
        let descriptor = [
            0x06, 0x89, 0xff, // Usage Page (Vendor 0xff89)
            0x09, 0x10, // Usage (0x10)
            0xa1, 0x01, // Collection (Application)
            0xc0, // End Collection
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x06, // Usage (Keyboard)
            0xa1, 0x01, // Collection (Application)
            0xc0, // End Collection
            0x05, 0x0c, // Usage Page (Consumer)
            0x09, 0x01, // Usage (Consumer Control)
            0xa1, 0x01, // Collection (Application)
            0xc0, // End Collection
        ];
        assert_eq!(
            parse_report_descriptor_usages(&descriptor),
            vec![(0xff89, 0x0010), (0x0001, 0x0006), (0x000c, 0x0001)]
        );
    }

    #[test]
    fn lamp_array_report_ids_are_extracted_per_collection() {
        // Minimal LampArray descriptor: an application collection containing
        // one logical collection per report, each with its own Report ID.
        let descriptor = [
            0x05, 0x59, // Usage Page (Lighting & Illumination)
            0x09, 0x01, // Usage (LampArray)
            0xa1, 0x01, // Collection (Application)
            0x09, 0x02, //   Usage (LampArrayAttributesReport)
            0xa1, 0x02, //   Collection (Logical)
            0x85, 0x01, //     Report ID (1)
            0x09, 0x03, //     Usage (LampCount)
            0xb1, 0x02, //     Feature
            0xc0, //   End Collection
            0x09, 0x70, //   Usage (LampArrayControlReport)
            0xa1, 0x02, //   Collection (Logical)
            0x85, 0x06, //     Report ID (6)
            0x09, 0x71, //     Usage (AutonomousMode)
            0xb1, 0x02, //     Feature
            0xc0, //   End Collection
            0xc0, // End Collection
        ];

        let ids = parse_lamp_array_report_ids(&descriptor);
        assert_eq!(ids.attributes, Some(1));
        assert_eq!(ids.control, Some(6));
        assert_eq!(ids.multi_update, None);
        assert_eq!(ids.range_update, None);
    }

    #[test]
    fn non_lamp_array_descriptor_yields_no_report_ids() {
        // A keyboard descriptor with a feature report must not be mistaken for
        // a LampArray report.
        let descriptor = [
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x06, // Usage (Keyboard)
            0xa1, 0x01, // Collection (Application)
            0x85, 0x02, //   Report ID (2)
            0xb1, 0x02, //   Feature
            0xc0, // End Collection
        ];

        assert_eq!(
            parse_lamp_array_report_ids(&descriptor),
            LampArrayReportIds::default()
        );
    }
}
