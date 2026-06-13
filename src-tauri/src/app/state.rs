use std::sync::Arc;

use crate::{
    domain::AppInfo,
    services::{
        device_service::DeviceService, diagnostics_service::DiagnosticsService,
        lighting_service::LightingService, permission_service::PermissionService,
        profile_service::ProfileService,
    },
};

pub struct AppState {
    pub app_info: AppInfo,
    pub device_service: Arc<DeviceService>,
    pub lighting_service: Arc<LightingService>,
    pub profile_service: Arc<ProfileService>,
    pub diagnostics_service: Arc<DiagnosticsService>,
    pub permission_service: Arc<PermissionService>,
}
