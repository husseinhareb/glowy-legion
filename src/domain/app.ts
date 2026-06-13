export interface AppInfo {
  name: string;
  version: string;
  backend_mode: string;
  real_hardware_writes_enabled: boolean;
  requires_user_caution: boolean;
  configuration_warnings: string[];
}
