import type { DiagnosticsReport } from "../../domain/diagnostics";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { formatDeviceFamily, formatEffect } from "../../shared/utils/formatting";

interface DiagnosticsTableProps {
  report: DiagnosticsReport;
}

export function DiagnosticsTable({ report }: DiagnosticsTableProps) {
  return (
    <div className="diagnostics-table">
      <div>
        <span>OS target</span>
        <strong>
          {report.os} / {report.architecture}
        </strong>
      </div>
      <div>
        <span>DMI vendor</span>
        <strong>{report.dmi_sys_vendor ?? "Unavailable"}</strong>
      </div>
      <div>
        <span>Detected laptop model</span>
        <strong>{report.dmi_product_name ?? report.detected_device.product_name}</strong>
      </div>
      <div>
        <span>DMI product version</span>
        <strong>{report.dmi_product_version ?? "Unavailable"}</strong>
      </div>
      <div>
        <span>Device family</span>
        <strong>{formatDeviceFamily(report.detected_device.family)}</strong>
      </div>
      <div>
        <span>Backend mode</span>
        <strong>{report.backend_mode}</strong>
      </div>
      <div>
        <span>Real hardware backend available</span>
        <strong>
          {report.real_hardware_backend_available ? "true" : "false"}
        </strong>
      </div>
      <div>
        <span>HID handle held open</span>
        <strong>
          {report.hid_device_opened ? "true" : "false (never at rest)"}
        </strong>
      </div>
      <div>
        <span>HID access disabled by safety flag</span>
        <strong>
          {report.hid_access_disabled_by_safety_flag ? "true" : "false"}
        </strong>
      </div>
      <div>
        <span>Safe RGB-control interfaces</span>
        <strong>{report.eligible_rgb_interface_count}</strong>
      </div>
      <div>
        <span>Writes enabled</span>
        <strong>
          {report.real_hardware_writes_enabled
            ? "true"
            : report.backend_mode === "lenovo-hid"
              ? "false - blocked by product/interface gate"
              : "false"}
        </strong>
      </div>
      <div>
        <span>Dry-run enabled</span>
        <strong>{report.dry_run_enabled ? "true" : "false"}</strong>
      </div>
      <div>
        <span>Experimental product ID override active</span>
        <strong>{report.experimental_override_active ? "true" : "false"}</strong>
      </div>
      <div>
        <span>Write allowlist source</span>
        <strong>{report.write_allowlist_source}</strong>
      </div>
      <div>
        <span>Running as root</span>
        <strong>{report.running_as_root ? "true" : "false"}</strong>
      </div>
      <div>
        <span>Likely permission issue</span>
        <strong>{report.likely_permission_issue ? "true" : "false"}</strong>
      </div>
      <div>
        <span>Supported effects</span>
        <strong>
          {report.supported_effects.map((effect) => formatEffect(effect)).join(", ")}
        </strong>
      </div>
      <div>
        <span>Unsupported effects</span>
        <strong>
          {report.unsupported_effects.length > 0
            ? report.unsupported_effects.map((effect) => formatEffect(effect)).join(", ")
            : "None"}
        </strong>
      </div>
      <div>
        <span>Capabilities</span>
        <strong>
          {report.capabilities.supports_zones
            ? `${report.capabilities.zone_count} zones`
            : "No RGB write capabilities"}
        </strong>
      </div>
      <div>
        <span>Write caution</span>
        <strong>
          <StatusBadge
            label={
              report.real_hardware_writes_enabled
                ? "Caution required"
                : report.backend_mode === "lenovo-hid"
                  ? "Writes blocked"
                  : "No write risk"
            }
            tone={report.real_hardware_writes_enabled ? "danger" : "ok"}
          />
        </strong>
      </div>
    </div>
  );
}
