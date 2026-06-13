//! Safe HID interface filtering.
//!
//! The same USB device (one VID/PID) exposes several HID interfaces: the real
//! keyboard input interface, a consumer/media-key interface, and a
//! vendor-defined RGB control interface. Opening the wrong one can break
//! keyboard input system-wide, so a VID/PID match alone is never enough to
//! open anything. Classification here is pure and uses only passive metadata.

use crate::{
    domain::HidInterfaceSummary,
    infrastructure::linux::hidraw::{
        HidrawInterfaceMetadata, USAGE_LAMP_ARRAY, USAGE_PAGE_LIGHTING,
    },
};

/// Generic Desktop usage page.
const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
/// Keyboard usage on the Generic Desktop page.
const USAGE_KEYBOARD: u16 = 0x06;
/// Keyboard/Keypad usage page.
const USAGE_PAGE_KEYBOARD: u16 = 0x07;
/// Consumer usage page (media keys).
const USAGE_PAGE_CONSUMER: u16 = 0x0c;
/// Start of the vendor-defined usage page range.
const USAGE_PAGE_VENDOR_START: u16 = 0xff00;

/// Classify one hidraw interface for safety. `is_rgb_candidate_device` must be
/// true only when the interface belongs to the detected Lenovo RGB candidate
/// device — eligibility is never granted to other hardware.
pub fn classify_hid_interface(
    metadata: &HidrawInterfaceMetadata,
    is_rgb_candidate_device: bool,
) -> HidInterfaceSummary {
    // Safety decisions look at EVERY top-level collection in the report
    // descriptor, not just the first: real composite interfaces (e.g. ITE
    // 048d:c693 interface 0) mix vendor-defined collections with the actual
    // keyboard input collection. One keyboard collection anywhere makes the
    // whole interface untouchable.
    let pairs = &metadata.all_usage_pairs;
    let is_keyboard_input = pairs.iter().any(|(page, usage)| {
        (*page == USAGE_PAGE_GENERIC_DESKTOP && *usage == USAGE_KEYBOARD)
            || *page == USAGE_PAGE_KEYBOARD
    });
    let is_consumer_control = pairs.iter().any(|(page, _)| *page == USAGE_PAGE_CONSUMER);
    let is_vendor_defined = pairs
        .iter()
        .any(|(page, _)| *page >= USAGE_PAGE_VENDOR_START);
    let is_lamp_array = pairs
        .iter()
        .any(|(page, usage)| *page == USAGE_PAGE_LIGHTING && *usage == USAGE_LAMP_ARRAY);
    let metadata_missing =
        pairs.is_empty() || metadata.usage_page.is_none() || metadata.usage.is_none();

    let (eligible_for_rgb_probe, safety_reason) = if is_keyboard_input {
        (
            false,
            "Carries a keyboard input collection. Opening it can break keyboard input; LegionGlow will never open it.".to_string(),
        )
    } else if metadata_missing {
        (
            false,
            "Usage metadata is missing, so the interface role cannot be verified. Unsafe to open by default.".to_string(),
        )
    } else if is_consumer_control {
        (
            false,
            "Consumer-control (media key) interface. Not opened unless proven necessary."
                .to_string(),
        )
    } else if is_lamp_array && is_rgb_candidate_device {
        (
            true,
            "Standard HID LampArray lighting interface on the detected Lenovo RGB candidate. Eligible for an explicit probe (read-only attributes).".to_string(),
        )
    } else if is_lamp_array {
        (
            false,
            "LampArray lighting interface, but not on the detected Lenovo RGB candidate device."
                .to_string(),
        )
    } else if !is_vendor_defined {
        (
            false,
            "Not a vendor-defined interface. Only vendor-defined usage pages (>= 0xff00) and HID LampArray interfaces are RGB-control candidates.".to_string(),
        )
    } else if !is_rgb_candidate_device {
        (
            false,
            "Vendor-defined, but not on the detected Lenovo RGB candidate device.".to_string(),
        )
    } else {
        (
            true,
            "Vendor-defined interface on the detected Lenovo RGB candidate. Eligible for an explicit probe.".to_string(),
        )
    };

    // Feature-report writes through hidraw do not detach the kernel driver and
    // cannot interfere with keyboard input, so composite keyboard+vendor
    // interfaces may receive ITE vendor lighting payloads even though they are
    // blocked for generic probing.
    let eligible_for_vendor_write =
        is_vendor_defined && is_rgb_candidate_device && !metadata_missing;

    HidInterfaceSummary {
        vendor_id: format!("0x{:04x}", metadata.vendor_id),
        product_id: format!("0x{:04x}", metadata.product_id),
        path: Some(metadata.dev_path.clone()).filter(|path| !path.is_empty()),
        interface_number: metadata.interface_number,
        usage_page: metadata.usage_page,
        usage: metadata.usage,
        manufacturer: metadata.manufacturer.clone(),
        product: metadata.product.clone(),
        is_keyboard_input,
        is_consumer_control,
        is_vendor_defined,
        is_lamp_array,
        eligible_for_rgb_probe,
        eligible_for_vendor_write,
        safety_reason,
    }
}

/// Pick the interface to receive ITE vendor feature-report writes.
///
/// Prefers non-keyboard interfaces (dedicated vendor page) when available.
/// Falls back to composite keyboard+vendor interfaces because hidraw feature
/// reports are routed by report ID inside the kernel and cannot detach
/// drivers or interrupt keyboard input.
pub fn select_vendor_write_target(
    interfaces: &[HidInterfaceSummary],
) -> Result<&HidInterfaceSummary, String> {
    let eligible: Vec<&HidInterfaceSummary> = interfaces
        .iter()
        .filter(|interface| interface.eligible_for_vendor_write && interface.path.is_some())
        .collect();

    match eligible.as_slice() {
        [] => Err(
            "No vendor-defined interface found for ITE write protocol. Cannot send lighting commands.".to_string(),
        ),
        [single] => Ok(single),
        multiple => {
            // Prefer dedicated (non-keyboard) vendor interface over composite.
            multiple
                .iter()
                .find(|interface| !interface.is_keyboard_input)
                .copied()
                .or_else(|| multiple.first().copied())
                .ok_or_else(|| "No eligible vendor write target".to_string())
        }
    }
}

/// Pick the single interface that an explicit user-triggered probe may open.
///
/// Returns an error unless exactly one eligible RGB-control interface with a
/// usable path exists. Ambiguity (zero or multiple candidates) blocks the
/// probe entirely; keyboard input interfaces are never considered.
pub fn select_probe_target(
    interfaces: &[HidInterfaceSummary],
) -> Result<&HidInterfaceSummary, String> {
    let eligible: Vec<&HidInterfaceSummary> = interfaces
        .iter()
        .filter(|interface| interface.eligible_for_rgb_probe && interface.path.is_some())
        .collect();

    match eligible.as_slice() {
        [single] => {
            // Defense in depth: refuse even if a classification bug ever marks
            // a keyboard interface eligible.
            if single.is_keyboard_input {
                return Err(
                    "Refusing to probe: the selected interface is a keyboard input interface."
                        .to_string(),
                );
            }
            Ok(single)
        }
        [] => Err(
            "No safe RGB-control HID interface was identified. LegionGlow will not open this device."
                .to_string(),
        ),
        _ => Err(format!(
            "{} interfaces look like RGB-control candidates. Probing is blocked until exactly one safe interface is identified.",
            eligible.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_hid_interface, select_probe_target};
    use crate::infrastructure::linux::hidraw::{HidrawInterfaceMetadata, LampArrayReportIds};

    fn metadata_with_pairs(
        usage_page: Option<u16>,
        usage: Option<u16>,
        all_usage_pairs: Vec<(u16, u16)>,
    ) -> HidrawInterfaceMetadata {
        HidrawInterfaceMetadata {
            dev_path: "/dev/hidraw9".to_string(),
            vendor_id: 0x048d,
            product_id: 0xc693,
            hid_name: Some("ITE Tech. Inc. ITE Device".to_string()),
            interface_number: Some(1),
            usage_page,
            usage,
            all_usage_pairs,
            manufacturer: Some("ITE Tech. Inc.".to_string()),
            product: Some("ITE Device".to_string()),
            lamp_array_reports: LampArrayReportIds::default(),
        }
    }

    fn metadata(usage_page: Option<u16>, usage: Option<u16>) -> HidrawInterfaceMetadata {
        let pairs = match (usage_page, usage) {
            (Some(page), Some(usage)) => vec![(page, usage)],
            _ => Vec::new(),
        };
        metadata_with_pairs(usage_page, usage, pairs)
    }

    #[test]
    fn generic_desktop_keyboard_is_marked_keyboard_and_never_eligible() {
        let summary = classify_hid_interface(&metadata(Some(0x01), Some(0x06)), true);

        assert!(summary.is_keyboard_input);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn keyboard_keypad_usage_page_is_marked_keyboard() {
        let summary = classify_hid_interface(&metadata(Some(0x07), Some(0x00)), true);

        assert!(summary.is_keyboard_input);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn consumer_control_is_not_eligible() {
        let summary = classify_hid_interface(&metadata(Some(0x0c), Some(0x01)), true);

        assert!(summary.is_consumer_control);
        assert!(!summary.is_keyboard_input);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn vendor_defined_interface_on_candidate_device_is_eligible() {
        let summary = classify_hid_interface(&metadata(Some(0xff89), Some(0x01)), true);

        assert!(summary.is_vendor_defined);
        assert!(summary.eligible_for_rgb_probe);
    }

    #[test]
    fn missing_usage_metadata_is_not_eligible_by_default() {
        let summary = classify_hid_interface(&metadata(None, None), true);

        assert!(!summary.eligible_for_rgb_probe);
        assert!(summary.safety_reason.contains("missing"));
    }

    #[test]
    fn vid_pid_match_alone_is_insufficient() {
        // Same VID/PID as the RGB candidate, but no usage metadata: the device
        // identity alone never makes an interface safe to open.
        let summary = classify_hid_interface(&metadata(None, None), true);
        assert!(!summary.eligible_for_rgb_probe);

        // Even a fully-described keyboard interface on the candidate device
        // stays blocked.
        let keyboard = classify_hid_interface(&metadata(Some(0x01), Some(0x06)), true);
        assert!(!keyboard.eligible_for_rgb_probe);
    }

    #[test]
    fn composite_interface_with_keyboard_collection_is_never_eligible() {
        // The real ITE 048d:c693 interface 0: the FIRST collection is
        // vendor-defined, but the same interface also carries the Generic
        // Desktop Keyboard collection (it backs the laptop's keyboard input).
        // The keyboard collection must poison the whole interface.
        let summary = classify_hid_interface(
            &metadata_with_pairs(
                Some(0xff89),
                Some(0x10),
                vec![
                    (0xff89, 0x0010),
                    (0xff89, 0x0007),
                    (0x0001, 0x0006),
                    (0x000c, 0x0001),
                ],
            ),
            true,
        );

        assert!(summary.is_keyboard_input);
        assert!(summary.is_vendor_defined);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn lamp_array_interface_on_candidate_device_is_probe_eligible() {
        // 048d:c693 interface 1 exposes the HID LampArray page (0x59): the
        // standardized lighting interface, fully separate from the keyboard
        // interface. It may be probed (read-only) on the candidate device.
        let summary = classify_hid_interface(&metadata(Some(0x59), Some(0x01)), true);

        assert!(!summary.is_keyboard_input);
        assert!(summary.is_lamp_array);
        assert!(summary.eligible_for_rgb_probe);
    }

    #[test]
    fn lamp_array_interface_on_other_device_is_not_eligible() {
        let summary = classify_hid_interface(&metadata(Some(0x59), Some(0x01)), false);

        assert!(summary.is_lamp_array);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn lamp_array_mixed_with_keyboard_collection_stays_blocked() {
        // Defense in depth: if a composite interface ever carried both a
        // LampArray collection and a keyboard collection, the keyboard
        // collection must win.
        let summary = classify_hid_interface(
            &metadata_with_pairs(
                Some(0x59),
                Some(0x01),
                vec![(0x0059, 0x0001), (0x0001, 0x0006)],
            ),
            true,
        );

        assert!(summary.is_keyboard_input);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn vendor_defined_interface_on_other_device_is_not_eligible() {
        let summary = classify_hid_interface(&metadata(Some(0xff00), Some(0x01)), false);

        assert!(summary.is_vendor_defined);
        assert!(!summary.eligible_for_rgb_probe);
    }

    #[test]
    fn probe_target_requires_exactly_one_eligible_interface() {
        let keyboard = classify_hid_interface(&metadata(Some(0x01), Some(0x06)), true);
        let vendor = classify_hid_interface(&metadata(Some(0xff89), Some(0x01)), true);

        assert!(select_probe_target(&[keyboard.clone()]).is_err());
        assert!(select_probe_target(&[]).is_err());
        assert!(select_probe_target(&[vendor.clone(), vendor.clone()]).is_err());

        let mixed = [keyboard, vendor.clone()];
        let selected = select_probe_target(&mixed).expect("one candidate");
        assert_eq!(selected, &vendor);
    }

    #[test]
    fn no_safe_interface_message_matches_ui_contract() {
        let error = select_probe_target(&[]).unwrap_err();
        assert!(error.contains("No safe RGB-control HID interface was identified"));
    }
}
