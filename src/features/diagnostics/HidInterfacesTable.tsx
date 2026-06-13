import type { HidInterfaceSummary } from "../../domain/diagnostics";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";

function hex(value: number | null, width: number): string {
  if (value === null) {
    return "—";
  }
  return `0x${value.toString(16).padStart(width, "0")}`;
}

function safetyLabel(iface: HidInterfaceSummary): string {
  if (iface.eligible_for_rgb_probe) {
    return iface.is_lamp_array ? "LampArray – probe eligible" : "Probe eligible";
  }
  if (iface.is_keyboard_input) {
    return "Keyboard – never opened";
  }
  if (iface.is_consumer_control) {
    return "Consumer – not opened";
  }
  if (iface.usage_page === null || iface.usage === null) {
    return "Unknown – not opened";
  }
  return "Not opened";
}

export function HidInterfacesTable({
  interfaces,
}: {
  interfaces: HidInterfaceSummary[];
}) {
  if (interfaces.length === 0) {
    return (
      <Notice tone="info">
        No HID interface metadata was found (or HID access is disabled by the
        safety flag).
      </Notice>
    );
  }

  return (
    <div className="hid-table hid-table--interfaces">
      <div className="hid-table__header hid-table__row--interfaces">
        <span>Device</span>
        <span>Iface</span>
        <span>Usage page</span>
        <span>Usage</span>
        <span>Safety</span>
        <span>Reason</span>
      </div>
      {interfaces.map((iface) => (
        <div
          className="hid-table__row hid-table__row--interfaces"
          key={`${iface.vendor_id}-${iface.product_id}-${iface.path ?? ""}-${iface.interface_number ?? ""}`}
        >
          <code title={iface.path ?? undefined}>
            {iface.vendor_id.replace("0x", "")}:{iface.product_id.replace("0x", "")}
          </code>
          <code>{iface.interface_number ?? "—"}</code>
          <code>{hex(iface.usage_page, 4)}</code>
          <code>{hex(iface.usage, 4)}</code>
          <StatusBadge
            label={safetyLabel(iface)}
            tone={
              iface.eligible_for_rgb_probe
                ? "ok"
                : iface.is_keyboard_input
                  ? "danger"
                  : "warn"
            }
          />
          <span title={iface.safety_reason}>{iface.safety_reason}</span>
        </div>
      ))}
    </div>
  );
}
