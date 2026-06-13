export type DeviceFamily =
  | "LenovoLegion"
  | "LenovoLoq"
  | "LenovoUnknown"
  | "Unsupported";

export interface DeviceCapabilities {
  supports_static: boolean;
  supports_breathing: boolean;
  supports_wave: boolean;
  supports_reactive: boolean;
  supports_rainbow: boolean;
  supports_brightness: boolean;
  supports_speed: boolean;
  supports_direction: boolean;
  supports_primary_color: boolean;
  supports_secondary_color: boolean;
  supports_zones: boolean;
  zone_count: number;
  supports_per_key_rgb: boolean;
}

export interface DeviceInfo {
  id: string;
  vendor: string;
  product_name: string;
  family: DeviceFamily;
  supported: boolean;
  backend: string;
  capabilities: DeviceCapabilities;
}
