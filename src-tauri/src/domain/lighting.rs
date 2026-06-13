use serde::{Deserialize, Serialize};

use crate::domain::device::DeviceCapabilities;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// A per-zone color override.
///
/// Zone indices are zero-based: zone 0 is the leftmost zone and the highest
/// valid index is `zone_count - 1`. A 4-zone keyboard therefore uses indices
/// `0..=3`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZoneColor {
    pub zone_index: u8,
    pub color: RgbColor,
}

impl ZoneColor {
    #[allow(dead_code)]
    pub const fn new(zone_index: u8, color: RgbColor) -> Self {
        Self { zone_index, color }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightingEffect {
    Static,
    Breathing,
    Wave,
    Reactive,
    Rainbow,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyboardState {
    pub effect: LightingEffect,
    pub primary_color: RgbColor,
    pub secondary_color: Option<RgbColor>,
    pub brightness: u8,
    pub speed: u8,
    pub direction: EffectDirection,
    pub enabled: bool,
    /// Optional per-zone colors. When present and complete (one entry per
    /// device zone with zero-based indices), zone-aware effects use these
    /// instead of `primary_color`. Defaults to `None` for backwards
    /// compatibility with older frontends.
    #[serde(default)]
    pub zone_colors: Option<Vec<ZoneColor>>,
}

impl KeyboardState {
    pub fn default_static() -> Self {
        Self {
            effect: LightingEffect::Static,
            primary_color: RgbColor::new(0, 148, 255),
            secondary_color: Some(RgbColor::new(0, 255, 210)),
            brightness: 70,
            speed: 45,
            direction: EffectDirection::LeftToRight,
            enabled: true,
            zone_colors: None,
        }
    }

    pub fn off() -> Self {
        Self {
            effect: LightingEffect::Off,
            primary_color: RgbColor::new(0, 0, 0),
            secondary_color: None,
            brightness: 0,
            speed: 0,
            direction: EffectDirection::LeftToRight,
            enabled: false,
            zone_colors: None,
        }
    }
}

#[allow(dead_code)]
pub fn all_lighting_effects() -> Vec<LightingEffect> {
    vec![
        LightingEffect::Static,
        LightingEffect::Breathing,
        LightingEffect::Wave,
        LightingEffect::Reactive,
        LightingEffect::Rainbow,
        LightingEffect::Off,
    ]
}

pub fn supported_lighting_effects(capabilities: &DeviceCapabilities) -> Vec<LightingEffect> {
    let mut effects = Vec::new();

    if capabilities.supports_static {
        effects.push(LightingEffect::Static);
    }
    if capabilities.supports_breathing {
        effects.push(LightingEffect::Breathing);
    }
    if capabilities.supports_wave {
        effects.push(LightingEffect::Wave);
    }
    if capabilities.supports_reactive {
        effects.push(LightingEffect::Reactive);
    }
    if capabilities.supports_rainbow {
        effects.push(LightingEffect::Rainbow);
    }

    effects.push(LightingEffect::Off);
    effects
}
