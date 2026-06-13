use crate::domain::UdevRulePreview;

/// Filename used for the LegionGlow-managed rule. Numbered high so it overrides
/// defaults. This is a constant — the frontend can never choose it.
pub const UDEV_RULE_FILENAME: &str = "99-legionglow-lenovo-rgb.rules";

/// Directory where system udev rules live.
pub const UDEV_RULES_DIR: &str = "/etc/udev/rules.d";

/// Absolute path of the LegionGlow-managed rule file. Constant, never derived
/// from untrusted input.
pub fn udev_rule_path() -> String {
    format!("{UDEV_RULES_DIR}/{UDEV_RULE_FILENAME}")
}

/// The exact rule line that gets installed and that "Copy udev rule" copies.
///
/// Targets the hidraw subsystem because hidapi uses the Linux hidraw backend
/// to open `/dev/hidrawN` directly. `ATTRS` (plural) walks the device tree
/// upward to match the parent USB device's idVendor/idProduct, so the rule
/// is specific to one vendor/product pair without touching any other device.
pub fn udev_rule_line(vendor_id: u16, product_id: u16) -> String {
    format!(
        "SUBSYSTEM==\"hidraw\", ATTRS{{idVendor}}==\"{vendor_id:04x}\", ATTRS{{idProduct}}==\"{product_id:04x}\", TAG+=\"uaccess\""
    )
}

/// USB-level alternative for documentation only. Grants access to the USB
/// device node but NOT to /dev/hidrawN, so it is not sufficient on its own.
pub fn usb_rule_line(vendor_id: u16, product_id: u16) -> String {
    format!(
        "SUBSYSTEM==\"usb\", ATTR{{idVendor}}==\"{vendor_id:04x}\", ATTR{{idProduct}}==\"{product_id:04x}\", TAG+=\"uaccess\""
    )
}

/// Strictly parse and normalize a `vendor:product` device id.
///
/// Accepts `048d:c693` and `0x048d:0xc693` (casing normalized). Rejects
/// wildcards, vendor-wide patterns, and the literal `all`. Used as a guard so
/// no command is ever built from a value that is not an exact hex pair.
pub fn normalize_device_id(input: &str) -> Result<(u16, u16), String> {
    let lowered = input.trim().to_ascii_lowercase();

    if lowered.is_empty() {
        return Err("empty device id".to_string());
    }

    if lowered == "all" || lowered.contains('*') {
        return Err(format!(
            "'{input}' is not allowed: wildcards and 'all' are rejected"
        ));
    }

    let (vendor, product) = lowered
        .split_once(':')
        .ok_or_else(|| format!("'{input}' is not in 'vendor:product' form"))?;

    let vendor_id = parse_hex_u16(vendor)
        .ok_or_else(|| format!("'{vendor}' is not a valid 4-digit hex vendor id"))?;
    let product_id = parse_hex_u16(product)
        .ok_or_else(|| format!("'{product}' is not a valid 4-digit hex product id"))?;

    Ok((vendor_id, product_id))
}

fn parse_hex_u16(value: &str) -> Option<u16> {
    let trimmed = value.trim();
    let digits = trimmed.strip_prefix("0x").unwrap_or(trimmed);

    if digits.is_empty() || digits.len() > 4 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    u16::from_str_radix(digits, 16).ok()
}

/// Manual terminal commands the user can copy and run themselves (no GUI sudo).
pub fn manual_install_commands(vendor_id: u16, product_id: u16) -> Vec<String> {
    let rule = udev_rule_line(vendor_id, product_id);
    let path = udev_rule_path();
    vec![
        format!("echo '{rule}' | sudo tee {path}"),
        "sudo udevadm control --reload-rules".to_string(),
        "sudo udevadm trigger".to_string(),
        "# Then reconnect the device (or relog/reboot) and restart LegionGlow.".to_string(),
    ]
}

/// Manual reload commands.
pub fn manual_reload_commands() -> Vec<String> {
    vec![
        "sudo udevadm control --reload-rules".to_string(),
        "sudo udevadm trigger".to_string(),
    ]
}

/// Build a non-installing preview of the conservative udev rule for the detected
/// Lenovo ITE RGB candidate. The `rule` field is exactly the single line that
/// gets installed and that "Copy udev rule" copies.
pub fn build_udev_rule_preview(vendor_id: u16, product_id: u16) -> UdevRulePreview {
    let rule = udev_rule_line(vendor_id, product_id);

    let explanation = format!(
        "This rule grants the active local session access to the {vendor_id:04x}:{product_id:04x} \
         hidraw device through systemd's uaccess tag, so LegionGlow can open it without root. It \
         is specific to vendor {vendor_id:04x} and product {product_id:04x} only — it does NOT \
         grant access to all Lenovo/ITE devices. LegionGlow never installs it without an explicit \
         confirmation in which you enter your system password."
    );

    let warnings = vec![
        "Review this rule before installing. LegionGlow does not install udev rules silently."
            .to_string(),
        "Installation runs through `sudo`: your password is used once for the install command and \
         is never stored or logged, and the GUI itself never runs as root."
            .to_string(),
        format!(
            "USB-level alternative (documentation only, insufficient on its own): {}",
            usb_rule_line(vendor_id, product_id)
        ),
    ];

    UdevRulePreview {
        available: true,
        vendor_id: format!("0x{vendor_id:04x}"),
        product_id: format!("0x{product_id:04x}"),
        rule,
        filename: UDEV_RULE_FILENAME.to_string(),
        explanation,
        install_commands: manual_install_commands(vendor_id, product_id),
        reload_commands: manual_reload_commands(),
        warnings,
    }
}

/// Preview returned when no candidate device has been detected.
pub fn unavailable_udev_rule_preview() -> UdevRulePreview {
    UdevRulePreview {
        available: false,
        vendor_id: String::new(),
        product_id: String::new(),
        rule: String::new(),
        filename: UDEV_RULE_FILENAME.to_string(),
        explanation:
            "No Lenovo ITE RGB candidate was detected, so no udev rule can be suggested yet."
                .to_string(),
        install_commands: Vec::new(),
        reload_commands: Vec::new(),
        warnings: vec![
            "Connect to a supported Lenovo Legion/LOQ keyboard and run diagnostics first."
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_udev_rule_preview, manual_install_commands, normalize_device_id, udev_rule_line,
        udev_rule_path, unavailable_udev_rule_preview, UDEV_RULE_FILENAME,
    };

    #[test]
    fn builds_uaccess_rule_for_specific_product() {
        let preview = build_udev_rule_preview(0x048d, 0xc693);

        assert!(preview.available);
        assert_eq!(
            preview.rule,
            "SUBSYSTEM==\"hidraw\", ATTRS{idVendor}==\"048d\", ATTRS{idProduct}==\"c693\", TAG+=\"uaccess\""
        );
        assert_eq!(preview.vendor_id, "0x048d");
        assert_eq!(preview.product_id, "0xc693");
        assert_eq!(preview.filename, "99-legionglow-lenovo-rgb.rules");
    }

    #[test]
    fn rule_does_not_use_vendor_wildcard() {
        let rule = udev_rule_line(0x048d, 0xc693);

        assert!(!rule.contains('*'));
        assert!(!rule.contains("c693*"));
        assert!(rule.contains("ATTRS{idProduct}==\"c693\""));
    }

    #[test]
    fn filename_and_path_are_constant_and_safe() {
        assert_eq!(UDEV_RULE_FILENAME, "99-legionglow-lenovo-rgb.rules");
        assert_eq!(
            udev_rule_path(),
            "/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules"
        );
        // No traversal or unexpected separators in the filename.
        assert!(!UDEV_RULE_FILENAME.contains('/'));
        assert!(!UDEV_RULE_FILENAME.contains(".."));
    }

    #[test]
    fn normalizes_plain_and_prefixed_ids() {
        assert_eq!(normalize_device_id("048d:c693"), Ok((0x048d, 0xc693)));
        assert_eq!(normalize_device_id("0x048D:0xC693"), Ok((0x048d, 0xc693)));
        assert_eq!(normalize_device_id("  048d:c693 "), Ok((0x048d, 0xc693)));
    }

    #[test]
    fn rejects_wildcards_and_all() {
        assert!(normalize_device_id("048d:*").is_err());
        assert!(normalize_device_id("*:*").is_err());
        assert!(normalize_device_id("all").is_err());
    }

    #[test]
    fn rejects_invalid_ids() {
        assert!(normalize_device_id("").is_err());
        assert!(normalize_device_id("048d").is_err());
        assert!(normalize_device_id("zzzz:c693").is_err());
        assert!(normalize_device_id("048d:c6933").is_err());
        assert!(normalize_device_id("048d c693").is_err());
    }

    #[test]
    fn manual_commands_target_constant_path() {
        let commands = manual_install_commands(0x048d, 0xc693);

        assert!(commands
            .iter()
            .any(|cmd| cmd.contains("/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules")));
        assert!(commands.iter().any(|cmd| cmd.contains("udevadm")));
    }

    #[test]
    fn unavailable_preview_is_marked_unavailable() {
        let preview = unavailable_udev_rule_preview();

        assert!(!preview.available);
        assert!(preview.install_commands.is_empty());
    }
}
