import type { DiagnosticsReport } from "../../domain/diagnostics";
import { Card } from "../../shared/components/Card";
import { StatusBadge } from "../../shared/components/StatusBadge";

interface ChecklistItem {
  label: string;
  /** undefined => informational step (no automatic state) */
  done?: boolean;
}

/**
 * Manual hardware validation checklist. Steps that the backend can confirm are
 * reflected automatically; the rest are manual reminders for safe real-hardware
 * bring-up of 048d:c693.
 */
export function ValidationChecklist({ report }: { report: DiagnosticsReport }) {
  const items: ChecklistItem[] = [
    {
      label: "Backend is dry-run (no real writes)",
      done: report.dry_run_enabled,
    },
    {
      label: "DMI model is LOQ 17IRX10 (or your expected model)",
      done: (report.dmi_product_version ?? "")
        .toUpperCase()
        .includes("LOQ 17IRX10"),
    },
    {
      label: "HID product ID 048d:c693 is detected",
      done: report.hid_devices.some(
        (device) =>
          device.vendor_id.toLowerCase().includes("048d") &&
          device.product_id.toLowerCase().includes("c693"),
      ),
    },
    {
      label: "Payload hex is generated",
      done: report.last_payload_hex !== null,
    },
    {
      label: "Exactly one safe RGB-control interface is identified",
      done: report.eligible_rgb_interface_count === 1,
    },
    {
      label:
        "Probe HID access manually (Settings → Fix permissions → Probe HID access)",
    },
    { label: "Add the udev rule manually if the device cannot be opened" },
    { label: "Reload udev rules and reconnect (or restart your session)" },
    { label: "Run real mode only after dry-run succeeds" },
    { label: "In real mode, send the safe test payload first" },
    { label: "Test Static red, green, blue" },
    { label: "Test brightness low and high" },
    { label: "Test Off" },
    { label: "Copy the diagnostics JSON for bug reports" },
  ];

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Safety</p>
          <h2>Manual hardware validation checklist</h2>
        </div>
      </div>
      <ul className="checklist">
        {items.map((item) => (
          <li className="checklist__item" key={item.label}>
            <StatusBadge
              label={
                item.done === undefined
                  ? "Manual"
                  : item.done
                    ? "OK"
                    : "Pending"
              }
              tone={
                item.done === undefined ? "warn" : item.done ? "ok" : "warn"
              }
            />
            <span>{item.label}</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
