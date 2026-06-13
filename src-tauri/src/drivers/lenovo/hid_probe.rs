use crate::{
    domain::{HidAccessProbe, HidOpenFailureKind, LampArrayAttributesSummary},
    drivers::lenovo::detection::LenovoHidDeviceInfo,
};

/// Classify why a detected HID device could not be opened.
///
/// hidapi sometimes returns a vague string (e.g. "hid_error is not implemented
/// yet"). In that case we only infer a permission problem when the device was
/// detected, opening failed, and we are running as a normal (non-root) user —
/// the exact situation where missing udev `uaccess` rules are the usual cause.
pub fn classify_hid_open_failure(
    raw_error: Option<&str>,
    detected: bool,
    can_open: bool,
    running_as_root: bool,
) -> Option<HidOpenFailureKind> {
    if can_open || !detected {
        return None;
    }

    let lowered = raw_error.unwrap_or_default().to_ascii_lowercase();

    if lowered.contains("permission")
        || lowered.contains("access")
        || lowered.contains("denied")
        || lowered.contains("eacces")
        || lowered.contains("operation not permitted")
    {
        return Some(HidOpenFailureKind::PermissionDenied);
    }

    if lowered.contains("busy") || lowered.contains("in use") || lowered.contains("ebusy") {
        return Some(HidOpenFailureKind::DeviceBusy);
    }

    if lowered.contains("no such device")
        || lowered.contains("cannot open")
        || lowered.contains("unable to open")
    {
        return Some(HidOpenFailureKind::BackendUnavailable);
    }

    // Vague or empty error string: only infer a permission issue for non-root.
    if !running_as_root {
        Some(HidOpenFailureKind::PermissionDenied)
    } else {
        Some(HidOpenFailureKind::Unknown)
    }
}

fn user_message_for(
    kind: Option<HidOpenFailureKind>,
    can_open: bool,
    lamp_array_attributes: Option<&LampArrayAttributesSummary>,
) -> String {
    if can_open {
        return match lamp_array_attributes {
            Some(attributes) => format!(
                "The LampArray lighting interface was opened briefly, its read-only attributes \
                 were read ({} lamps, kind: {}), and it was closed again. No lighting state was \
                 changed; this does not enable writes.",
                attributes.lamp_count, attributes.kind_label
            ),
            None => "The RGB-control interface was opened briefly and closed again. Access works; \
                     this does not enable writes."
                .to_string(),
        };
    }

    match kind {
        Some(HidOpenFailureKind::PermissionDenied) => {
            "The device was detected but could not be opened. This is almost always a permission \
             problem: your user lacks access to the raw HID device."
                .to_string()
        }
        Some(HidOpenFailureKind::DeviceBusy) => {
            "The device was detected but is currently busy, likely held open by another process or \
             driver."
                .to_string()
        }
        Some(HidOpenFailureKind::BackendUnavailable) => {
            "The device was detected in the list but the HID backend could not open its path."
                .to_string()
        }
        Some(HidOpenFailureKind::UnsupportedProduct) => {
            "This product is detected but not enabled for opening on this backend.".to_string()
        }
        Some(HidOpenFailureKind::Unknown) | None => {
            "The device was detected but could not be opened, and the HID backend did not provide a \
             specific reason."
                .to_string()
        }
    }
}

fn recommended_action_for(kind: Option<HidOpenFailureKind>, can_open: bool) -> String {
    if can_open {
        return "No action needed.".to_string();
    }

    match kind {
        Some(HidOpenFailureKind::PermissionDenied) => {
            "Install the udev rule from Settings → Fix permissions (it reloads and re-triggers the \
             device for you). If access is still blocked, log out and back in or reboot — an \
             internal keyboard cannot be replugged. Do not run LegionGlow as root."
                .to_string()
        }
        Some(HidOpenFailureKind::DeviceBusy) => {
            "Close other RGB tools that may hold the device, then run diagnostics again."
                .to_string()
        }
        Some(HidOpenFailureKind::BackendUnavailable) => {
            "Reconnect the device and re-run diagnostics. If it persists, check kernel HID support."
                .to_string()
        }
        Some(HidOpenFailureKind::UnsupportedProduct) => {
            "Enable the product via the experimental allowlist only after dry-run validation."
                .to_string()
        }
        Some(HidOpenFailureKind::Unknown) | None => {
            "Try the udev rule preview first, then re-run diagnostics. Avoid running as root."
                .to_string()
        }
    }
}

/// Build a structured access probe for a detected device.
pub fn build_hid_access_probe(
    device: &LenovoHidDeviceInfo,
    can_open: bool,
    raw_error: Option<&str>,
    running_as_root: bool,
    lamp_array_attributes: Option<LampArrayAttributesSummary>,
) -> HidAccessProbe {
    let failure_kind = classify_hid_open_failure(raw_error, true, can_open, running_as_root);

    HidAccessProbe {
        vendor_id: format!("0x{:04x}", device.vendor_id),
        product_id: format!("0x{:04x}", device.product_id),
        label: device.label.clone(),
        manufacturer: device.manufacturer_string.clone(),
        product: device.product_string.clone(),
        path_available: !device.path.is_empty(),
        can_open,
        failure_kind,
        raw_error: raw_error.map(ToString::to_string),
        user_message: user_message_for(failure_kind, can_open, lamp_array_attributes.as_ref()),
        recommended_action: recommended_action_for(failure_kind, can_open),
        lamp_array_attributes,
    }
}

#[cfg(test)]
mod tests {
    use super::classify_hid_open_failure;
    use crate::domain::HidOpenFailureKind;

    #[test]
    fn open_success_has_no_failure() {
        assert_eq!(classify_hid_open_failure(None, true, true, false), None);
    }

    #[test]
    fn explicit_permission_error_is_classified() {
        assert_eq!(
            classify_hid_open_failure(Some("Permission denied (os error 13)"), true, false, false),
            Some(HidOpenFailureKind::PermissionDenied)
        );
    }

    #[test]
    fn busy_error_is_classified() {
        assert_eq!(
            classify_hid_open_failure(Some("device or resource busy"), true, false, false),
            Some(HidOpenFailureKind::DeviceBusy)
        );
    }

    #[test]
    fn vague_error_infers_permission_for_non_root() {
        assert_eq!(
            classify_hid_open_failure(Some("hid_error is not implemented yet"), true, false, false),
            Some(HidOpenFailureKind::PermissionDenied)
        );
    }

    #[test]
    fn vague_error_stays_unknown_for_root() {
        assert_eq!(
            classify_hid_open_failure(Some("hid_error is not implemented yet"), true, false, true),
            Some(HidOpenFailureKind::Unknown)
        );
    }

    #[test]
    fn undetected_device_has_no_failure() {
        assert_eq!(
            classify_hid_open_failure(Some("permission denied"), false, false, false),
            None
        );
    }
}
