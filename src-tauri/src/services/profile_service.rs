use std::sync::Arc;

use crate::{
    app::error::AppError,
    domain::{DeviceCapabilities, KeyboardState, LightingProfile},
    infrastructure::storage::profile_repository::ProfileRepository,
    services::lighting_service::LightingService,
    utils::validation::compute_profile_compatibility,
};

#[derive(Clone)]
pub struct ProfileService {
    lighting_service: Arc<LightingService>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl ProfileService {
    pub fn new(
        lighting_service: Arc<LightingService>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            lighting_service,
            profile_repository,
        }
    }

    pub fn list_builtin_profiles(&self) -> Result<Vec<LightingProfile>, AppError> {
        let capabilities = self.active_capabilities();
        let mut profiles = self.profile_repository.list_builtin_profiles()?;

        for profile in &mut profiles {
            profile.compatibility = compute_profile_compatibility(&profile.state, &capabilities);
        }

        Ok(profiles)
    }

    pub fn apply_profile(&self, profile_id: &str) -> Result<KeyboardState, AppError> {
        let profile = self.profile_repository.find_builtin_profile(profile_id)?;
        let capabilities = self.active_capabilities();
        let compatibility = compute_profile_compatibility(&profile.state, &capabilities);

        if !compatibility.supported {
            return Err(AppError::UnsupportedEffect(profile.state.effect.clone()));
        }

        self.lighting_service.set_keyboard_state(profile.state)
    }

    /// Capabilities of the active device, falling back to "unsupported" so
    /// profiles are conservatively marked incompatible when detection fails.
    fn active_capabilities(&self) -> DeviceCapabilities {
        self.lighting_service
            .detect_device()
            .map(|device| device.capabilities)
            .unwrap_or_else(|_| DeviceCapabilities::unsupported())
    }
}
