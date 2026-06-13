export const APP_DISPLAY_NAME = "LegionGlow";

export const MOCK_BACKEND_NOTICE =
  "Mock backend active. No hardware writes.";

export const LENOVO_HID_DRY_RUN_NOTICE =
  "Lenovo HID dry-run active. Device detection and payload generation are real, but no hardware writes are sent.";

export const LENOVO_HID_BACKEND_NOTICE =
  "Experimental Lenovo HID backend active. Real hardware writes are enabled.";

export function backendModeNotice(backendMode?: string): string {
  switch (backendMode) {
    case "lenovo-hid-dry-run":
      return LENOVO_HID_DRY_RUN_NOTICE;
    case "lenovo-hid":
      return LENOVO_HID_BACKEND_NOTICE;
    case "mock":
    default:
      return MOCK_BACKEND_NOTICE;
  }
}
