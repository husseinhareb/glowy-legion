use crate::{
    app::error::AppError,
    domain::{DeviceCapabilities, DeviceInfo, KeyboardState, LightingEffect, ProfileCompatibility},
};

pub fn ensure_device_supported(device: &DeviceInfo) -> Result<(), AppError> {
    if device.supported {
        Ok(())
    } else {
        Err(AppError::UnsupportedDevice(device.product_name.clone()))
    }
}

/// Drop optional state the device cannot use so it degrades gracefully instead
/// of failing. The 4-zone Lenovo protocol ignores secondary color, so a profile
/// that carries one (e.g. Gaming Red) is still applicable — the extra color is
/// simply discarded rather than treated as an error.
pub fn coerce_state_to_capabilities(
    mut state: KeyboardState,
    capabilities: &DeviceCapabilities,
) -> KeyboardState {
    if !capabilities.supports_secondary_color {
        state.secondary_color = None;
    }

    if !capabilities.supports_zones || validate_zone_colors(&state, capabilities).is_err() {
        state.zone_colors = None;
    }

    state
}

pub fn normalize_keyboard_state(mut state: KeyboardState) -> Result<KeyboardState, AppError> {
    if state.brightness > 100 {
        return Err(AppError::InvalidBrightness(state.brightness));
    }

    if state.speed > 100 {
        return Err(AppError::InvalidSpeed(state.speed));
    }

    if state.effect == LightingEffect::Off {
        state.enabled = false;
        state.brightness = 0;
        state.speed = 0;
        state.secondary_color = None;
        return Ok(state);
    }

    if state.brightness == 0 {
        state.effect = LightingEffect::Off;
        state.brightness = 0;
        state.speed = 0;
        state.secondary_color = None;
        state.enabled = false;
        return Ok(state);
    }

    state.enabled = true;
    Ok(state)
}

pub fn validate_keyboard_state(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> Result<(), AppError> {
    validate_rgb_capabilities(state, capabilities)?;

    if state.brightness > 100 {
        return Err(AppError::InvalidBrightness(state.brightness));
    }

    if state.speed > 100 {
        return Err(AppError::InvalidSpeed(state.speed));
    }

    validate_effect_support(&state.effect, capabilities)?;

    if !capabilities.supports_brightness && state.brightness > 0 {
        return Err(AppError::InvalidEffect {
            effect: state.effect.clone(),
            reason: "the detected device does not expose brightness control".to_string(),
        });
    }

    if requires_speed(&state.effect) && !capabilities.supports_speed {
        return Err(AppError::InvalidEffect {
            effect: state.effect.clone(),
            reason: "the detected device does not expose speed control".to_string(),
        });
    }

    if requires_direction(&state.effect) && !capabilities.supports_direction {
        return Err(AppError::InvalidEffect {
            effect: state.effect.clone(),
            reason: "the detected device does not expose direction control".to_string(),
        });
    }

    validate_zone_colors(state, capabilities)?;

    Ok(())
}

/// Validate optional per-zone colors against the device.
///
/// Zone indices are zero-based (`0..zone_count`). The set must not contain
/// duplicate indices, exceed the device zone count, or contain out-of-range
/// indices.
pub fn validate_zone_colors(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> Result<(), AppError> {
    let zones = match &state.zone_colors {
        Some(zones) if !zones.is_empty() => zones,
        _ => return Ok(()),
    };

    if !capabilities.supports_zones {
        return Err(AppError::InvalidEffect {
            effect: state.effect.clone(),
            reason: "the detected device does not support per-zone colors".to_string(),
        });
    }

    if zones.len() > capabilities.zone_count as usize {
        return Err(AppError::InvalidColor(format!(
            "received {} zone colors but the device only has {} zones",
            zones.len(),
            capabilities.zone_count
        )));
    }

    let mut seen = Vec::with_capacity(zones.len());
    for zone in zones {
        if zone.zone_index >= capabilities.zone_count {
            return Err(AppError::InvalidColor(format!(
                "zone index {} is out of range for a {}-zone device",
                zone.zone_index, capabilities.zone_count
            )));
        }

        if seen.contains(&zone.zone_index) {
            return Err(AppError::InvalidColor(format!(
                "duplicate zone index {}",
                zone.zone_index
            )));
        }

        seen.push(zone.zone_index);
    }

    Ok(())
}

/// Compute whether a profile can be applied to a device, collecting all reasons
/// it cannot (rather than failing on the first). The backend remains the final
/// authority on profile compatibility.
pub fn compute_profile_compatibility(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> ProfileCompatibility {
    let mut reasons = Vec::new();

    if validate_effect_support(&state.effect, capabilities).is_err() {
        reasons.push(format!(
            "Effect {:?} is not supported by the active device.",
            state.effect
        ));
    }

    if state.brightness > 0 && !capabilities.supports_brightness {
        reasons.push("The active device does not expose brightness control.".to_string());
    }

    if requires_speed(&state.effect) && !capabilities.supports_speed {
        reasons.push("The active device does not expose speed control.".to_string());
    }

    if requires_direction(&state.effect) && !capabilities.supports_direction {
        reasons.push("The active device does not expose direction control.".to_string());
    }

    // Secondary color and per-zone colors are optional decoration: a device that
    // does not support them ignores them (see `coerce_state_to_capabilities`),
    // so they never make a profile incompatible.

    if reasons.is_empty() {
        ProfileCompatibility::supported()
    } else {
        ProfileCompatibility::unsupported(reasons)
    }
}

fn validate_rgb_capabilities(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
) -> Result<(), AppError> {
    if !capabilities.supports_primary_color
        && state.effect != LightingEffect::Off
        && (state.primary_color.r > 0 || state.primary_color.g > 0 || state.primary_color.b > 0)
    {
        return Err(AppError::InvalidColor(
            "primary color is not supported by this device".to_string(),
        ));
    }

    if state.secondary_color.is_some() && !capabilities.supports_secondary_color {
        return Err(AppError::InvalidColor(
            "secondary color is not supported by this device".to_string(),
        ));
    }

    Ok(())
}

fn validate_effect_support(
    effect: &LightingEffect,
    capabilities: &DeviceCapabilities,
) -> Result<(), AppError> {
    let supported = match effect {
        LightingEffect::Static => capabilities.supports_static,
        LightingEffect::Breathing => capabilities.supports_breathing,
        LightingEffect::Wave => capabilities.supports_wave,
        LightingEffect::Reactive => capabilities.supports_reactive,
        LightingEffect::Rainbow => capabilities.supports_rainbow,
        LightingEffect::Off => true,
    };

    if supported {
        Ok(())
    } else {
        Err(AppError::InvalidEffect {
            effect: effect.clone(),
            reason: "the detected device does not report this capability".to_string(),
        })
    }
}

fn requires_speed(effect: &LightingEffect) -> bool {
    matches!(
        effect,
        LightingEffect::Breathing
            | LightingEffect::Wave
            | LightingEffect::Reactive
            | LightingEffect::Rainbow
    )
}

fn requires_direction(effect: &LightingEffect) -> bool {
    matches!(effect, LightingEffect::Wave | LightingEffect::Rainbow)
}

#[cfg(test)]
mod tests {
    use crate::{
        app::error::AppError,
        domain::{DeviceCapabilities, KeyboardState, LightingEffect},
    };

    use super::{
        compute_profile_compatibility, normalize_keyboard_state, validate_keyboard_state,
        validate_zone_colors,
    };
    use crate::domain::{RgbColor, ZoneColor};

    fn four_zone_colors() -> Vec<ZoneColor> {
        vec![
            ZoneColor::new(0, RgbColor::new(255, 0, 0)),
            ZoneColor::new(1, RgbColor::new(0, 255, 0)),
            ZoneColor::new(2, RgbColor::new(0, 0, 255)),
            ZoneColor::new(3, RgbColor::new(255, 255, 0)),
        ]
    }

    #[test]
    fn accepts_valid_four_zone_colors() {
        let mut state = KeyboardState::default_static();
        state.zone_colors = Some(four_zone_colors());

        assert!(validate_zone_colors(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).is_ok());
    }

    #[test]
    fn rejects_duplicate_zone_indices() {
        let mut state = KeyboardState::default_static();
        state.zone_colors = Some(vec![
            ZoneColor::new(0, RgbColor::new(1, 1, 1)),
            ZoneColor::new(0, RgbColor::new(2, 2, 2)),
        ]);

        assert!(validate_zone_colors(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).is_err());
    }

    #[test]
    fn rejects_out_of_range_zone_index() {
        let mut state = KeyboardState::default_static();
        state.zone_colors = Some(vec![ZoneColor::new(7, RgbColor::new(1, 1, 1))]);

        assert!(validate_zone_colors(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).is_err());
    }

    #[test]
    fn rejects_more_zone_colors_than_device_zones() {
        let mut state = KeyboardState::default_static();
        let mut colors = four_zone_colors();
        colors.push(ZoneColor::new(0, RgbColor::new(9, 9, 9)));
        state.zone_colors = Some(colors);

        assert!(validate_zone_colors(&state, &DeviceCapabilities::lenovo_4_zone_rgb()).is_err());
    }

    #[test]
    fn breathing_profile_supported_when_breathing_supported() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Breathing;
        state.secondary_color = None;

        let compatibility =
            compute_profile_compatibility(&state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(compatibility.supported);
    }

    #[test]
    fn reactive_profile_unsupported_without_reactive_capability() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Reactive;

        let compatibility =
            compute_profile_compatibility(&state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(!compatibility.supported);
        assert!(!compatibility.reasons.is_empty());
    }

    #[test]
    fn profile_with_secondary_color_is_supported_on_device_without_secondary() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Breathing;
        state.secondary_color = Some(RgbColor::new(255, 126, 39));

        // lenovo_4_zone_rgb does NOT support secondary color, but it is optional.
        let compatibility =
            compute_profile_compatibility(&state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(compatibility.supported);
    }

    #[test]
    fn coercion_drops_unsupported_secondary_and_zone_colors() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = Some(RgbColor::new(1, 2, 3));
        state.zone_colors = Some(four_zone_colors());

        let mut caps = DeviceCapabilities::lenovo_4_zone_rgb();
        caps.supports_secondary_color = false;
        caps.supports_zones = false;

        let coerced = super::coerce_state_to_capabilities(state, &caps);

        assert!(coerced.secondary_color.is_none());
        assert!(coerced.zone_colors.is_none());
    }

    #[test]
    fn coercion_keeps_valid_zone_colors_when_supported() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.zone_colors = Some(four_zone_colors());

        let coerced =
            super::coerce_state_to_capabilities(state, &DeviceCapabilities::lenovo_4_zone_rgb());

        assert!(coerced.zone_colors.is_some());
    }

    #[test]
    fn off_profile_always_supported() {
        let compatibility = compute_profile_compatibility(
            &KeyboardState::off(),
            &DeviceCapabilities::unsupported(),
        );

        assert!(compatibility.supported);
    }

    #[test]
    fn normalizes_off_effect() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Off;
        state.brightness = 80;
        state.enabled = true;

        let normalized = normalize_keyboard_state(state).unwrap();

        assert_eq!(normalized.effect, LightingEffect::Off);
        assert_eq!(normalized.brightness, 0);
        assert!(!normalized.enabled);
    }

    #[test]
    fn brightness_zero_normalizes_to_off() {
        let mut state = KeyboardState::default_static();
        state.brightness = 0;
        state.enabled = true;

        let normalized = normalize_keyboard_state(state).unwrap();

        assert_eq!(normalized.effect, LightingEffect::Off);
        assert!(!normalized.enabled);
        assert_eq!(normalized.speed, 0);
    }

    #[test]
    fn non_off_with_brightness_enables_state() {
        let mut state = KeyboardState::default_static();
        state.enabled = false;
        state.brightness = 20;

        let normalized = normalize_keyboard_state(state).unwrap();

        assert_eq!(normalized.effect, LightingEffect::Static);
        assert!(normalized.enabled);
    }

    #[test]
    fn rejects_out_of_range_brightness() {
        let mut state = KeyboardState::default_static();
        state.brightness = 101;

        let result = normalize_keyboard_state(state);

        assert!(matches!(result, Err(AppError::InvalidBrightness(101))));
    }

    #[test]
    fn rejects_out_of_range_speed() {
        let mut state = KeyboardState::default_static();
        state.speed = 101;

        let result = normalize_keyboard_state(state);

        assert!(matches!(result, Err(AppError::InvalidSpeed(101))));
    }

    #[test]
    fn rejects_unsupported_effect() {
        let mut state = KeyboardState::default_static();
        state.effect = LightingEffect::Reactive;

        let result = validate_keyboard_state(&state, &DeviceCapabilities::mock_loq());

        assert!(result.is_err());
    }
}
