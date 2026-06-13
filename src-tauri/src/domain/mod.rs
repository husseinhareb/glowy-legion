pub mod app;
pub mod device;
pub mod diagnostics;
pub mod lighting;
pub mod permissions;
pub mod profile;

pub use app::AppInfo;
pub use device::{DeviceCapabilities, DeviceFamily, DeviceInfo};
pub use diagnostics::{
    DiagnosticsReport, HidAccessProbe, HidDeviceSummary, HidInterfaceSummary, HidOpenFailureKind,
    HidPayloadPreview, LampArrayAttributesSummary, UdevRulePreview, WriteAllowlistSource,
};
pub use lighting::{
    all_lighting_effects, supported_lighting_effects, EffectDirection, KeyboardState,
    LightingEffect, RgbColor,
};
pub use permissions::PermissionSetupResult;
// Part of `KeyboardState`'s public surface; referenced by tests and frontend mirror.
#[allow(unused_imports)]
pub use lighting::ZoneColor;
pub use profile::{LightingProfile, ProfileCompatibility};
