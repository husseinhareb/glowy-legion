#[derive(Debug, Default)]
pub struct SettingsRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    Mock,
    LenovoHidDryRun,
    LenovoHid,
}

impl BackendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendMode::Mock => "mock",
            BackendMode::LenovoHidDryRun => "lenovo-hid-dry-run",
            BackendMode::LenovoHid => "lenovo-hid",
        }
    }

    pub fn real_hardware_writes_enabled(self) -> bool {
        self == BackendMode::LenovoHid
    }

    pub fn requires_user_caution(self) -> bool {
        matches!(self, BackendMode::LenovoHidDryRun | BackendMode::LenovoHid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSelection {
    pub mode: BackendMode,
    pub warnings: Vec<String>,
}

/// Result of parsing `LEGIONGLOW_EXPERIMENTAL_ALLOW_PRODUCT_IDS`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExperimentalAllowlist {
    /// Explicitly allowed `(vendor_id, product_id)` pairs.
    pub entries: Vec<(u16, u16)>,
    /// Warnings about rejected or malformed entries.
    pub warnings: Vec<String>,
}

impl ExperimentalAllowlist {
    pub fn contains(&self, vendor_id: u16, product_id: u16) -> bool {
        self.entries
            .iter()
            .any(|(v, p)| *v == vendor_id && *p == product_id)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl SettingsRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn backend_selection(&self) -> BackendSelection {
        backend_selection_from_env(
            std::env::var("LEGIONGLOW_BACKEND").ok(),
            cfg!(debug_assertions),
        )
    }

    pub fn experimental_allowlist(&self) -> ExperimentalAllowlist {
        parse_experimental_allow_product_ids(
            std::env::var("LEGIONGLOW_EXPERIMENTAL_ALLOW_PRODUCT_IDS")
                .ok()
                .as_deref(),
        )
    }

    /// Emergency safety flag: when set, no HID library code that can open or
    /// claim devices runs, and detection falls back to DMI/sysfs-only data.
    pub fn hid_access_disabled(&self) -> bool {
        hid_access_disabled_from_env(std::env::var("LEGIONGLOW_DISABLE_HID").ok().as_deref())
    }
}

/// Parse `LEGIONGLOW_DISABLE_HID`. Any non-empty value other than `0`, `false`,
/// `no`, or `off` disables HID access — the flag errs on the side of safety.
pub fn hid_access_disabled_from_env(value: Option<&str>) -> bool {
    match value {
        None => false,
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            !(normalized.is_empty()
                || normalized == "0"
                || normalized == "false"
                || normalized == "no"
                || normalized == "off")
        }
    }
}

/// Parse the comma-separated experimental product ID override.
///
/// Accepts entries like `048d:c693` or `0x048d:0xc693`, normalizes casing, and
/// rejects wildcards (`*`), vendor-wide patterns (`048d:*`), and the literal
/// `all`.
pub fn parse_experimental_allow_product_ids(value: Option<&str>) -> ExperimentalAllowlist {
    let mut allowlist = ExperimentalAllowlist::default();

    let raw = match value {
        Some(raw) if !raw.trim().is_empty() => raw,
        _ => return allowlist,
    };

    for entry in raw.split(',') {
        let token = entry.trim();
        if token.is_empty() {
            continue;
        }

        let lowered = token.to_ascii_lowercase();

        if lowered == "all" || lowered.contains('*') {
            allowlist.warnings.push(format!(
                "Rejected experimental product ID override '{token}': wildcards and 'all' are not permitted."
            ));
            continue;
        }

        match parse_single_product_id(&lowered) {
            Some(pair) => {
                if !allowlist.entries.contains(&pair) {
                    allowlist.entries.push(pair);
                }
            }
            None => allowlist.warnings.push(format!(
                "Ignored malformed experimental product ID override '{token}'. Expected 'vendor:product', e.g. '048d:c693'."
            )),
        }
    }

    allowlist
}

fn parse_single_product_id(token: &str) -> Option<(u16, u16)> {
    let (vendor, product) = token.split_once(':')?;
    let vendor_id = parse_hex_u16(vendor)?;
    let product_id = parse_hex_u16(product)?;
    Some((vendor_id, product_id))
}

fn parse_hex_u16(value: &str) -> Option<u16> {
    let trimmed = value.trim();
    let digits = trimmed.strip_prefix("0x").unwrap_or(trimmed);

    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    u16::from_str_radix(digits, 16).ok()
}

fn backend_selection_from_env(value: Option<String>, debug_build: bool) -> BackendSelection {
    match value {
        Some(value) if value.eq_ignore_ascii_case("mock") => BackendSelection {
            mode: BackendMode::Mock,
            warnings: Vec::new(),
        },
        Some(value) if value.eq_ignore_ascii_case("lenovo-hid-dry-run") => BackendSelection {
            mode: BackendMode::LenovoHidDryRun,
            warnings: Vec::new(),
        },
        Some(value) if value.eq_ignore_ascii_case("lenovo-hid") => BackendSelection {
            mode: BackendMode::LenovoHid,
            warnings: Vec::new(),
        },
        Some(value) => BackendSelection {
            mode: default_backend_mode(debug_build),
            warnings: vec![format!(
                "Unknown LEGIONGLOW_BACKEND value '{value}'. Falling back to {} mode.",
                default_backend_mode(debug_build).as_str()
            )],
        },
        None => BackendSelection {
            mode: default_backend_mode(debug_build),
            warnings: Vec::new(),
        },
    }
}

fn default_backend_mode(debug_build: bool) -> BackendMode {
    if debug_build {
        BackendMode::LenovoHidDryRun
    } else {
        BackendMode::Mock
    }
}

#[cfg(test)]
mod tests {
    use super::{
        backend_selection_from_env, hid_access_disabled_from_env,
        parse_experimental_allow_product_ids, BackendMode,
    };

    #[test]
    fn hid_disable_flag_is_off_by_default() {
        assert!(!hid_access_disabled_from_env(None));
        assert!(!hid_access_disabled_from_env(Some("")));
        assert!(!hid_access_disabled_from_env(Some("0")));
        assert!(!hid_access_disabled_from_env(Some("false")));
        assert!(!hid_access_disabled_from_env(Some("off")));
    }

    #[test]
    fn hid_disable_flag_accepts_any_truthy_value() {
        assert!(hid_access_disabled_from_env(Some("1")));
        assert!(hid_access_disabled_from_env(Some("true")));
        assert!(hid_access_disabled_from_env(Some("yes")));
        // Unknown values err on the side of safety and disable HID access.
        assert!(hid_access_disabled_from_env(Some("anything")));
    }

    #[test]
    fn parses_single_product_id() {
        let allowlist = parse_experimental_allow_product_ids(Some("048d:c693"));

        assert_eq!(allowlist.entries, vec![(0x048d, 0xc693)]);
        assert!(allowlist.warnings.is_empty());
    }

    #[test]
    fn parses_multiple_and_prefixed_product_ids_with_normalized_casing() {
        let allowlist = parse_experimental_allow_product_ids(Some("0x048D:0xC693, 048d:c993"));

        assert_eq!(allowlist.entries, vec![(0x048d, 0xc693), (0x048d, 0xc993)]);
    }

    #[test]
    fn override_allows_specific_product() {
        let allowlist = parse_experimental_allow_product_ids(Some("048d:c693"));

        assert!(allowlist.contains(0x048d, 0xc693));
        assert!(!allowlist.contains(0x048d, 0xc994));
    }

    #[test]
    fn rejects_wildcard_overrides() {
        let allowlist = parse_experimental_allow_product_ids(Some("048d:*"));

        assert!(allowlist.entries.is_empty());
        assert_eq!(allowlist.warnings.len(), 1);
    }

    #[test]
    fn rejects_all_keyword() {
        let allowlist = parse_experimental_allow_product_ids(Some("all"));

        assert!(allowlist.entries.is_empty());
        assert_eq!(allowlist.warnings.len(), 1);
    }

    #[test]
    fn empty_value_yields_empty_allowlist() {
        assert!(parse_experimental_allow_product_ids(None).is_empty());
        assert!(parse_experimental_allow_product_ids(Some("   ")).is_empty());
    }

    #[test]
    fn ignores_malformed_entries_but_keeps_valid_ones() {
        let allowlist = parse_experimental_allow_product_ids(Some("garbage, 048d:c693"));

        assert_eq!(allowlist.entries, vec![(0x048d, 0xc693)]);
        assert_eq!(allowlist.warnings.len(), 1);
    }

    #[test]
    fn debug_build_defaults_to_lenovo_hid_dry_run() {
        let selection = backend_selection_from_env(None, true);

        assert_eq!(selection.mode, BackendMode::LenovoHidDryRun);
    }

    #[test]
    fn release_build_defaults_to_mock() {
        let selection = backend_selection_from_env(None, false);

        assert_eq!(selection.mode, BackendMode::Mock);
    }

    #[test]
    fn explicit_mock_overrides_debug_default() {
        let selection = backend_selection_from_env(Some("mock".to_string()), true);

        assert_eq!(selection.mode, BackendMode::Mock);
    }

    #[test]
    fn invalid_value_falls_back_to_default_with_warning() {
        let selection = backend_selection_from_env(Some("bad".to_string()), true);

        assert_eq!(selection.mode, BackendMode::LenovoHidDryRun);
        assert_eq!(selection.warnings.len(), 1);
    }
}
