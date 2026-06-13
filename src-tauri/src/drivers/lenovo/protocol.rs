use crate::{
    app::error::AppError,
    domain::{
        DeviceCapabilities, EffectDirection, HidPayloadPreview, KeyboardState, LightingEffect,
        RgbColor,
    },
};

pub const FEATURE_REPORT_LEN: usize = 33;
pub const REPORT_PREFIX_0: u8 = 0xcc;
pub const REPORT_PREFIX_1: u8 = 0x16;
pub const EFFECT_BYTE: usize = 2;
pub const SPEED_BYTE: usize = 3;
pub const BRIGHTNESS_BYTE: usize = 4;
pub const RGB_ZONE_START: usize = 5;
pub const RGB_ZONE_LEN: usize = 12;
pub const DIRECTION_BYTE_0: usize = 18;
pub const DIRECTION_BYTE_1: usize = 19;
const ZONE_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEffect {
    Static = 0x01,
    Breathing = 0x03,
    Wave = 0x04,
    Smooth = 0x06,
}

pub fn build_feature_report(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> Result<[u8; FEATURE_REPORT_LEN], AppError> {
    validate_protocol_capabilities(state, capabilities)?;
    validate_direction(state)?;

    let hardware_effect = hardware_effect_for_state(state)?;
    let mut payload = [0_u8; FEATURE_REPORT_LEN];
    payload[0] = REPORT_PREFIX_0;
    payload[1] = REPORT_PREFIX_1;
    payload[2] = hardware_effect as u8;
    payload[3] = map_speed_0_100_to_hardware(state.speed);
    payload[4] = map_brightness_0_100_to_hardware(state.brightness);

    let zone_bytes = build_zone_rgb_bytes(state);
    payload[RGB_ZONE_START..RGB_ZONE_START + RGB_ZONE_LEN].copy_from_slice(&zone_bytes);

    let direction = map_direction_to_hardware(state);
    payload[DIRECTION_BYTE_0] = direction;
    payload[DIRECTION_BYTE_1] = direction;

    Ok(payload)
}

/// Lowest-risk payload for verifying real hardware access: a static, very dim
/// blue at the minimum hardware brightness level. Used by the "safe first
/// write" flow before any animations are attempted.
pub fn build_safe_test_state() -> KeyboardState {
    KeyboardState {
        effect: LightingEffect::Static,
        // Dim blue is preferred over full brightness for the first real write.
        primary_color: RgbColor::new(0, 0, 32),
        secondary_color: None,
        // Maps to hardware brightness level 1 (the minimum non-off level).
        brightness: 1,
        speed: 0,
        direction: EffectDirection::LeftToRight,
        enabled: true,
        zone_colors: None,
    }
}

pub fn map_speed_0_100_to_hardware(speed: u8) -> u8 {
    match speed {
        0..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        _ => 4,
    }
}

pub fn map_brightness_0_100_to_hardware(brightness: u8) -> u8 {
    match brightness {
        0..=50 => 1,
        _ => 2,
    }
}

pub fn build_zone_rgb_bytes(state: &KeyboardState) -> [u8; RGB_ZONE_LEN] {
    let zone_colors = resolve_zone_colors(state);

    let mut bytes = [0_u8; RGB_ZONE_LEN];
    for (zone, color) in zone_colors.iter().enumerate() {
        let offset = zone * 3;
        bytes[offset] = color.r;
        bytes[offset + 1] = color.g;
        bytes[offset + 2] = color.b;
    }

    bytes
}

/// Resolve the effective color for each of the four hardware zones.
///
/// Off / disabled / zero-brightness states are always black. Per-zone colors
/// are honored only for zone-aware effects (Static and Breathing) when a
/// complete set of four zero-based indices (0..=3) is supplied; otherwise the
/// primary color fills every zone. Animated effects (Wave / Rainbow) ignore
/// per-zone colors at the protocol level.
fn resolve_zone_colors(state: &KeyboardState) -> [RgbColor; ZONE_COUNT] {
    if state.effect == LightingEffect::Off || !state.enabled || state.brightness == 0 {
        return [RgbColor::new(0, 0, 0); ZONE_COUNT];
    }

    let zone_aware = matches!(
        state.effect,
        LightingEffect::Static | LightingEffect::Breathing
    );

    if zone_aware {
        if let Some(per_zone) = complete_zone_palette(state) {
            return per_zone;
        }
    }

    [
        state.primary_color.clone(),
        state.primary_color.clone(),
        state.primary_color.clone(),
        state.primary_color.clone(),
    ]
}

/// Return a full 4-zone palette only when `zone_colors` provides exactly one
/// valid entry per zone index 0..=3 with no duplicates.
fn complete_zone_palette(state: &KeyboardState) -> Option<[RgbColor; ZONE_COUNT]> {
    let zones = state.zone_colors.as_ref()?;
    if zones.len() != ZONE_COUNT {
        return None;
    }

    let mut palette: [Option<RgbColor>; ZONE_COUNT] = [None, None, None, None];
    for zone in zones {
        let index = zone.zone_index as usize;
        if index >= ZONE_COUNT || palette[index].is_some() {
            return None;
        }
        palette[index] = Some(zone.color.clone());
    }

    Some([
        palette[0].clone()?,
        palette[1].clone()?,
        palette[2].clone()?,
        palette[3].clone()?,
    ])
}

/// Decode a generated feature report into a human-readable preview for the
/// diagnostics UI.
pub fn decode_payload_preview(payload: &[u8]) -> HidPayloadPreview {
    let byte_at = |index: usize| payload.get(index).copied();
    let hex_byte = |value: Option<u8>| {
        value
            .map(|byte| format!("0x{byte:02x}"))
            .unwrap_or_else(|| "n/a".to_string())
    };

    let header_bytes = payload
        .iter()
        .take(2)
        .map(|byte| format!("0x{byte:02x}"))
        .collect::<Vec<_>>();

    let mut zone_bytes = Vec::new();
    for zone in 0..ZONE_COUNT {
        let offset = RGB_ZONE_START + zone * 3;
        match (
            payload.get(offset),
            payload.get(offset + 1),
            payload.get(offset + 2),
        ) {
            (Some(r), Some(g), Some(b)) => {
                zone_bytes.push(format!("zone {}: {:02x} {:02x} {:02x}", zone + 1, r, g, b));
            }
            _ => break,
        }
    }

    let direction_bytes = [DIRECTION_BYTE_0, DIRECTION_BYTE_1]
        .into_iter()
        .filter_map(|index| payload.get(index).map(|byte| format!("0x{byte:02x}")))
        .collect::<Vec<_>>();

    HidPayloadPreview {
        length: payload.len(),
        hex: format_payload_hex(payload),
        header_bytes,
        effect_byte: hex_byte(byte_at(EFFECT_BYTE)),
        speed_byte: hex_byte(byte_at(SPEED_BYTE)),
        brightness_byte: hex_byte(byte_at(BRIGHTNESS_BYTE)),
        zone_bytes,
        direction_bytes,
        decoded_effect: decode_effect_byte(byte_at(EFFECT_BYTE)),
    }
}

fn decode_effect_byte(value: Option<u8>) -> String {
    match value {
        Some(byte) if byte == HardwareEffect::Static as u8 => "Static".to_string(),
        Some(byte) if byte == HardwareEffect::Breathing as u8 => "Breathing".to_string(),
        Some(byte) if byte == HardwareEffect::Wave as u8 => "Wave".to_string(),
        Some(byte) if byte == HardwareEffect::Smooth as u8 => "Rainbow (smooth)".to_string(),
        Some(byte) => format!("Unknown (0x{byte:02x})"),
        None => "n/a".to_string(),
    }
}

fn format_payload_hex(payload: &[u8]) -> String {
    payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_protocol_capabilities(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> Result<(), AppError> {
    if !capabilities.supports_zones || capabilities.zone_count != ZONE_COUNT as u8 {
        return Err(AppError::UnsupportedDevice(
            "Lenovo HID backend currently supports only 4-zone RGB keyboards".to_string(),
        ));
    }

    if state.secondary_color.is_some() && !capabilities.supports_secondary_color {
        return Err(AppError::InvalidColor(
            "secondary color is not supported by this Lenovo HID protocol".to_string(),
        ));
    }

    Ok(())
}

fn hardware_effect_for_state(state: &KeyboardState) -> Result<HardwareEffect, AppError> {
    if state.effect == LightingEffect::Off || !state.enabled || state.brightness == 0 {
        return Ok(HardwareEffect::Static);
    }

    match state.effect {
        LightingEffect::Static => Ok(HardwareEffect::Static),
        LightingEffect::Breathing => Ok(HardwareEffect::Breathing),
        LightingEffect::Wave => Ok(HardwareEffect::Wave),
        LightingEffect::Rainbow => Ok(HardwareEffect::Smooth),
        LightingEffect::Reactive => Err(AppError::UnsupportedEffect(LightingEffect::Reactive)),
        LightingEffect::Off => Ok(HardwareEffect::Static),
    }
}

fn validate_direction(state: &KeyboardState) -> Result<(), AppError> {
    if matches!(state.effect, LightingEffect::Wave | LightingEffect::Rainbow)
        && matches!(
            state.direction,
            EffectDirection::TopToBottom | EffectDirection::BottomToTop
        )
    {
        return Err(AppError::InvalidEffect {
            effect: state.effect.clone(),
            reason: "vertical wave directions are not enabled for the Lenovo HID protocol yet"
                .to_string(),
        });
    }

    Ok(())
}

fn map_direction_to_hardware(state: &KeyboardState) -> u8 {
    match state.direction {
        EffectDirection::LeftToRight => 0x01,
        EffectDirection::RightToLeft => 0x02,
        EffectDirection::TopToBottom | EffectDirection::BottomToTop => 0x01,
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{DeviceCapabilities, EffectDirection, KeyboardState, LightingEffect};

    use super::{
        build_feature_report, build_safe_test_state, build_zone_rgb_bytes, decode_payload_preview,
        map_brightness_0_100_to_hardware, map_speed_0_100_to_hardware, HardwareEffect,
    };
    use crate::domain::{RgbColor, ZoneColor};

    #[test]
    fn static_uses_per_zone_colors_when_complete() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.zone_colors = Some(vec![
            ZoneColor::new(0, RgbColor::new(255, 0, 0)),
            ZoneColor::new(1, RgbColor::new(0, 255, 0)),
            ZoneColor::new(2, RgbColor::new(0, 0, 255)),
            ZoneColor::new(3, RgbColor::new(255, 255, 0)),
        ]);

        let bytes = build_zone_rgb_bytes(&state);

        assert_eq!(bytes, [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]);
    }

    #[test]
    fn incomplete_zone_colors_fall_back_to_primary() {
        let mut state = KeyboardState::default_static();
        state.primary_color = RgbColor::new(10, 20, 30);
        state.secondary_color = None;
        state.zone_colors = Some(vec![
            ZoneColor::new(0, RgbColor::new(255, 0, 0)),
            ZoneColor::new(1, RgbColor::new(0, 255, 0)),
        ]);

        let bytes = build_zone_rgb_bytes(&state);

        assert_eq!(bytes, [10, 20, 30, 10, 20, 30, 10, 20, 30, 10, 20, 30]);
    }

    #[test]
    fn wave_ignores_per_zone_colors() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Wave;
        state.primary_color = RgbColor::new(1, 2, 3);
        state.secondary_color = None;
        state.zone_colors = Some(vec![
            ZoneColor::new(0, RgbColor::new(255, 0, 0)),
            ZoneColor::new(1, RgbColor::new(0, 255, 0)),
            ZoneColor::new(2, RgbColor::new(0, 0, 255)),
            ZoneColor::new(3, RgbColor::new(255, 255, 0)),
        ]);

        let bytes = build_zone_rgb_bytes(&state);

        assert_eq!(bytes, [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3]);
    }

    #[test]
    fn safe_test_state_is_dim_static_blue() {
        let state = build_safe_test_state();

        assert_eq!(state.effect, LightingEffect::Static);
        assert_eq!(state.primary_color, RgbColor::new(0, 0, 32));
        assert_eq!(map_brightness_0_100_to_hardware(state.brightness), 1);
    }

    #[test]
    fn decodes_payload_preview_fields() {
        let mut state = KeyboardState::default_static();
        state.primary_color = RgbColor::new(255, 0, 0);
        state.secondary_color = None;
        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        let preview = decode_payload_preview(&report);

        assert_eq!(preview.length, 33);
        assert_eq!(preview.header_bytes, vec!["0xcc", "0x16"]);
        assert_eq!(preview.effect_byte, "0x01");
        assert_eq!(preview.decoded_effect, "Static");
        assert_eq!(preview.zone_bytes.len(), 4);
        assert_eq!(preview.direction_bytes.len(), 2);
        assert!(preview.zone_bytes[0].contains("ff 00 00"));
    }

    #[test]
    fn maps_app_speed_to_four_hardware_steps() {
        assert_eq!(map_speed_0_100_to_hardware(0), 1);
        assert_eq!(map_speed_0_100_to_hardware(26), 2);
        assert_eq!(map_speed_0_100_to_hardware(51), 3);
        assert_eq!(map_speed_0_100_to_hardware(76), 4);
    }

    #[test]
    fn maps_app_brightness_to_two_hardware_steps() {
        assert_eq!(map_brightness_0_100_to_hardware(0), 1);
        assert_eq!(map_brightness_0_100_to_hardware(50), 1);
        assert_eq!(map_brightness_0_100_to_hardware(51), 2);
    }

    #[test]
    fn payload_length_and_header_are_stable() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report.len(), 33);
        assert_eq!(report[0], 0xcc);
        assert_eq!(report[1], 0x16);
    }

    #[test]
    fn static_red_fills_all_four_zones() {
        let mut state = KeyboardState::default_static();
        state.primary_color = crate::domain::RgbColor::new(255, 0, 0);
        state.secondary_color = None;

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report[2], HardwareEffect::Static as u8);
        assert_eq!(
            &report[5..17],
            &[255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0]
        );
    }

    #[test]
    fn breathing_blue_maps_to_breathing_effect() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Breathing;
        state.primary_color = crate::domain::RgbColor::new(0, 0, 255);
        state.secondary_color = None;

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report[2], HardwareEffect::Breathing as u8);
        assert_eq!(
            &report[5..17],
            &[0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255]
        );
    }

    #[test]
    fn wave_maps_to_wave_effect() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Wave;
        state.secondary_color = None;

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report[2], HardwareEffect::Wave as u8);
    }

    #[test]
    fn static_uses_primary_color_for_all_zones() {
        let state = KeyboardState::default_static();

        let bytes = build_zone_rgb_bytes(&state);

        assert_eq!(bytes, [0, 148, 255, 0, 148, 255, 0, 148, 255, 0, 148, 255]);
    }

    #[test]
    fn off_sends_static_black_report() {
        let state = KeyboardState::off();

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report[0], 0xcc);
        assert_eq!(report[1], 0x16);
        assert_eq!(report[2], HardwareEffect::Static as u8);
        assert_eq!(&report[5..17], &[0_u8; 12]);
    }

    #[test]
    fn rainbow_maps_to_smooth_effect() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Rainbow;
        state.secondary_color = None;
        state.direction = EffectDirection::RightToLeft;

        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        assert_eq!(report[2], HardwareEffect::Smooth as u8);
        assert_eq!(report[18], 0x02);
        assert_eq!(report[19], 0x02);
    }

    #[test]
    fn reactive_is_not_supported_yet() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Reactive;
        state.secondary_color = None;

        let result = build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(result.is_err());
    }

    #[test]
    fn unsupported_wave_direction_returns_validation_error() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Wave;
        state.direction = EffectDirection::TopToBottom;
        state.secondary_color = None;

        let result = build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(result.is_err());
    }

    #[test]
    fn formats_payload_as_lowercase_hex() {
        let state = KeyboardState::off();
        let report =
            build_feature_report(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).unwrap();

        let hex = decode_payload_preview(&report).hex;

        assert!(hex.starts_with("cc 16 01 01 01 00 00 00"));
        assert_eq!(hex.split(' ').count(), 33);
    }
}
