import type { DeviceInfo } from "./device";
import type { DeviceCapabilities } from "./device";
import type { LightingEffect } from "./lighting";

export interface HidDeviceSummary {
  vendor_id: string;
  product_id: string;
  manufacturer: string | null;
  product: string | null;
  path: string | null;
  known: boolean;
  supported_for_writes: boolean;
}

/** Safety classification of one HID interface, from passive sysfs metadata. */
export interface HidInterfaceSummary {
  vendor_id: string;
  product_id: string;
  path: string | null;
  interface_number: number | null;
  usage_page: number | null;
  usage: number | null;
  manufacturer: string | null;
  product: string | null;
  is_keyboard_input: boolean;
  is_consumer_control: boolean;
  is_vendor_defined: boolean;
  /** Standard HID LampArray lighting interface (usage page 0x59). */
  is_lamp_array: boolean;
  eligible_for_rgb_probe: boolean;
  safety_reason: string;
}

/** Read-only LampArray attributes read during a manual probe. */
export interface LampArrayAttributesSummary {
  lamp_count: number;
  lamp_array_kind: number;
  kind_label: string;
  min_update_interval_microseconds: number;
  bounding_box_width_micrometers: number;
  bounding_box_height_micrometers: number;
  bounding_box_depth_micrometers: number;
}

export type HidOpenFailureKind =
  | "PermissionDenied"
  | "DeviceBusy"
  | "BackendUnavailable"
  | "UnsupportedProduct"
  | "Unknown";

export interface HidAccessProbe {
  vendor_id: string;
  product_id: string;
  label: string;
  manufacturer: string | null;
  product: string | null;
  path_available: boolean;
  can_open: boolean;
  failure_kind: HidOpenFailureKind | null;
  raw_error: string | null;
  user_message: string;
  recommended_action: string;
  lamp_array_attributes: LampArrayAttributesSummary | null;
}

export interface UdevRulePreview {
  available: boolean;
  vendor_id: string;
  product_id: string;
  rule: string;
  filename: string;
  explanation: string;
  install_commands: string[];
  reload_commands: string[];
  warnings: string[];
}

export interface HidPayloadPreview {
  length: number;
  hex: string;
  header_bytes: string[];
  effect_byte: string;
  speed_byte: string;
  brightness_byte: string;
  zone_bytes: string[];
  direction_bytes: string[];
  decoded_effect: string;
}

export interface DiagnosticsReport {
  os: string;
  architecture: string;
  backend_mode: string;
  dmi_sys_vendor: string | null;
  dmi_product_name: string | null;
  dmi_product_version: string | null;
  detected_device: DeviceInfo;
  hid_devices: HidDeviceSummary[];
  hid_interfaces: HidInterfaceSummary[];
  eligible_rgb_interface_count: number;
  hid_access_disabled_by_safety_flag: boolean;
  known_supported_lenovo_rgb_device: HidDeviceSummary | null;
  hid_device_opened: boolean;
  hid_access_probe: HidAccessProbe | null;
  supported_effects: LightingEffect[];
  unsupported_effects: LightingEffect[];
  capabilities: DeviceCapabilities;
  real_hardware_backend_available: boolean;
  real_hardware_writes_enabled: boolean;
  dry_run_enabled: boolean;
  experimental_override_active: boolean;
  write_allowlist_source: string;
  requires_user_caution: boolean;
  likely_permission_issue: boolean;
  running_as_root: boolean;
  last_payload_hex: string | null;
  payload_preview: HidPayloadPreview | null;
  notes: string[];
  warnings: string[];
}
