use serde::{Deserialize, Serialize};

use crate::domain::lighting::KeyboardState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileCompatibility {
    pub supported: bool,
    pub reasons: Vec<String>,
}

impl ProfileCompatibility {
    pub fn supported() -> Self {
        Self {
            supported: true,
            reasons: Vec::new(),
        }
    }

    pub fn unsupported(reasons: Vec<String>) -> Self {
        Self {
            supported: false,
            reasons,
        }
    }
}

impl Default for ProfileCompatibility {
    fn default() -> Self {
        Self::supported()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightingProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: KeyboardState,
    /// Whether this profile can be applied to the active device. Computed by
    /// the backend against the detected capabilities; defaults to "supported"
    /// when not provided so older payloads still deserialize.
    #[serde(default)]
    pub compatibility: ProfileCompatibility,
}
