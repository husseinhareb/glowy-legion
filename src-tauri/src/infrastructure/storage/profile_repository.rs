use crate::{
    app::error::AppError,
    domain::{
        EffectDirection, KeyboardState, LightingEffect, LightingProfile, ProfileCompatibility,
        RgbColor,
    },
    utils::color,
};

pub trait ProfileRepository: Send + Sync {
    fn list_builtin_profiles(&self) -> Result<Vec<LightingProfile>, AppError>;
    fn find_builtin_profile(&self, profile_id: &str) -> Result<LightingProfile, AppError>;
}

#[derive(Debug, Default)]
pub struct BuiltinProfileRepository;

impl BuiltinProfileRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ProfileRepository for BuiltinProfileRepository {
    fn list_builtin_profiles(&self) -> Result<Vec<LightingProfile>, AppError> {
        Ok(builtin_profiles())
    }

    fn find_builtin_profile(&self, profile_id: &str) -> Result<LightingProfile, AppError> {
        builtin_profiles()
            .into_iter()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| AppError::ProfileNotFound(profile_id.to_string()))
    }
}

fn builtin_profiles() -> Vec<LightingProfile> {
    vec![
        LightingProfile {
            id: "calm-blue".to_string(),
            name: "Calm Blue".to_string(),
            description: "Low-distraction static blue for focused work.".to_string(),
            state: KeyboardState {
                effect: LightingEffect::Static,
                primary_color: color::CALM_BLUE,
                secondary_color: None,
                brightness: 45,
                speed: 20,
                direction: EffectDirection::LeftToRight,
                enabled: true,
                zone_colors: None,
            },
            compatibility: ProfileCompatibility::default(),
        },
        LightingProfile {
            id: "gaming-red".to_string(),
            name: "Gaming Red".to_string(),
            description: "Bright red breathing effect for high-contrast sessions.".to_string(),
            state: KeyboardState {
                effect: LightingEffect::Breathing,
                primary_color: color::GAMING_RED,
                secondary_color: Some(RgbColor::new(255, 126, 39)),
                brightness: 85,
                speed: 64,
                direction: EffectDirection::LeftToRight,
                enabled: true,
                zone_colors: None,
            },
            compatibility: ProfileCompatibility::default(),
        },
        LightingProfile {
            id: "matrix-green".to_string(),
            name: "Matrix Green".to_string(),
            description: "Reactive green lighting tuned for terminal-heavy workflows.".to_string(),
            state: KeyboardState {
                effect: LightingEffect::Reactive,
                primary_color: color::MATRIX_GREEN,
                secondary_color: Some(RgbColor::new(0, 84, 39)),
                brightness: 72,
                speed: 58,
                direction: EffectDirection::TopToBottom,
                enabled: true,
                zone_colors: None,
            },
            compatibility: ProfileCompatibility::default(),
        },
        LightingProfile {
            id: "rainbow-wave".to_string(),
            name: "Rainbow Wave".to_string(),
            description: "Animated rainbow sweep across supported keyboard zones.".to_string(),
            state: KeyboardState {
                effect: LightingEffect::Rainbow,
                primary_color: color::RAINBOW_PRIMARY,
                secondary_color: Some(color::RAINBOW_SECONDARY),
                brightness: 78,
                speed: 70,
                direction: EffectDirection::LeftToRight,
                enabled: true,
                zone_colors: None,
            },
            compatibility: ProfileCompatibility::default(),
        },
        LightingProfile {
            id: "backlight-off".to_string(),
            name: "Backlight Off".to_string(),
            description: "Disable keyboard lighting for battery-sensitive use.".to_string(),
            state: KeyboardState::off(),
            compatibility: ProfileCompatibility::default(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{BuiltinProfileRepository, ProfileRepository};

    #[test]
    fn finds_builtin_profile_by_id() {
        let repository = BuiltinProfileRepository::new();

        let profile = repository.find_builtin_profile("calm-blue").unwrap();

        assert_eq!(profile.name, "Calm Blue");
    }

    #[test]
    fn returns_profile_not_found() {
        let repository = BuiltinProfileRepository::new();

        let result = repository.find_builtin_profile("missing-profile");

        assert!(result.is_err());
    }
}
