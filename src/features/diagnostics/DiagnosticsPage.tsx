import { useEffect, useState } from "react";

import { runDiagnostics } from "../../api/diagnosticsApi";
import type {
  DiagnosticsReport,
  HidAccessProbe,
  HidDeviceSummary,
  HidPayloadPreview,
} from "../../domain/diagnostics";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { LoadingState } from "../../shared/components/LoadingState";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { useAsync } from "../../shared/hooks/useAsync";
import { DiagnosticsTable } from "./DiagnosticsTable";
import { HidInterfacesTable } from "./HidInterfacesTable";
import { ValidationChecklist } from "./ValidationChecklist";

export function DiagnosticsPage() {
  const diagnostics = useAsync<DiagnosticsReport>();
  const [copyStatus, setCopyStatus] = useState<string | null>(null);

  useEffect(() => {
    diagnostics.run(runDiagnostics).catch(() => undefined);
  }, [diagnostics.run]);

  const report = diagnostics.value;

  const copyDiagnostics = async () => {
    if (!report) {
      return;
    }
    await navigator.clipboard.writeText(JSON.stringify(report, null, 2));
    setCopyStatus("Diagnostics JSON copied to clipboard.");
  };

  return (
    <section className="page-stack">
      <div className="page-heading">
        <div>
          <p className="eyebrow">Diagnostics</p>
          <h1>Backend report</h1>
        </div>
        <div className="action-row">
          <Button
            disabled={diagnostics.loading}
            onClick={() => diagnostics.run(runDiagnostics).catch(() => undefined)}
          >
            Run diagnostics
          </Button>
          <Button disabled={!report} onClick={copyDiagnostics}>
            Copy JSON
          </Button>
        </div>
      </div>

      {diagnostics.error && <Notice tone="error">{diagnostics.error}</Notice>}
      {copyStatus && <Notice tone="success">{copyStatus}</Notice>}

      {diagnostics.loading && !report && (
        <LoadingState label="Running diagnostics" />
      )}

      {report && (
        <>
          {report.hid_access_disabled_by_safety_flag && (
            <Notice tone="warning">HID access disabled by safety flag.</Notice>
          )}

          <Card>
            <DiagnosticsTable report={report} />
            <HidDevicesTable devices={report.hid_devices} />
          </Card>

          <Card>
            <div className="card__header">
              <div>
                <p className="eyebrow">HID interfaces</p>
                <h2>Interface safety classification</h2>
              </div>
              <StatusBadge
                label={
                  report.eligible_rgb_interface_count === 1
                    ? "1 safe RGB interface"
                    : `${report.eligible_rgb_interface_count} safe RGB interfaces`
                }
                tone={report.eligible_rgb_interface_count === 1 ? "ok" : "warn"}
              />
            </div>
            <Notice tone="info">
              Diagnostics are passive and never open HID devices. Probing is
              manual: use “Probe HID access” in Settings → Fix permissions. It
              briefly opens only the single eligible RGB-control interface and
              sends no RGB data.
            </Notice>
            {report.eligible_rgb_interface_count !== 1 &&
              !report.hid_access_disabled_by_safety_flag && (
                <Notice tone="warning">
                  No safe RGB-control HID interface was identified. LegionGlow
                  will not open this device.
                </Notice>
              )}
            <HidInterfacesTable interfaces={report.hid_interfaces} />
          </Card>

          {report.hid_access_probe && (
            <HidAccessProbeView probe={report.hid_access_probe} />
          )}

          {report.payload_preview && (
            <PayloadPreviewView preview={report.payload_preview} />
          )}
          {!report.payload_preview && report.last_payload_hex && (
            <PayloadHexView hex={report.last_payload_hex} />
          )}

          <ValidationChecklist report={report} />

          <Notice tone="info">
            To fix HID permissions, install the udev rule, or run the safe test
            payload, open Settings → staged setup.
          </Notice>

          {report.warnings.length > 0 && (
            <Card>
              <h2>Warnings</h2>
              <div className="notes-list">
                {report.warnings.map((warning) => (
                  <Notice key={warning} tone="warning">
                    {warning}
                  </Notice>
                ))}
              </div>
            </Card>
          )}

          {report.notes.length > 0 && (
            <Card>
              <h2>Notes</h2>
              <div className="notes-list">
                {report.notes.map((note) => (
                  <Notice key={note}>{note}</Notice>
                ))}
              </div>
            </Card>
          )}
        </>
      )}
    </section>
  );
}

function HidAccessProbeView({ probe }: { probe: HidAccessProbe }) {
  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">HID access probe</p>
          <h2>
            {probe.vendor_id}:{probe.product_id}
          </h2>
        </div>
        <StatusBadge
          label={probe.can_open ? "Opened" : (probe.failure_kind ?? "Failed")}
          tone={probe.can_open ? "ok" : "danger"}
        />
      </div>
      <div className="diagnostics-table">
        <div>
          <span>Device</span>
          <strong>{probe.product ?? probe.label}</strong>
        </div>
        <div>
          <span>Manufacturer</span>
          <strong>{probe.manufacturer ?? "Unavailable"}</strong>
        </div>
        <div>
          <span>Path discovered</span>
          <strong>{probe.path_available ? "true" : "false"}</strong>
        </div>
        <div>
          <span>Can open</span>
          <strong>{probe.can_open ? "true" : "false"}</strong>
        </div>
        <div>
          <span>Failure category</span>
          <strong>{probe.failure_kind ?? "None"}</strong>
        </div>
        {probe.raw_error && (
          <div>
            <span>Raw error</span>
            <strong>{probe.raw_error}</strong>
          </div>
        )}
      </div>
      <Notice tone={probe.can_open ? "success" : "warning"}>
        {probe.user_message}
      </Notice>
      <Notice tone="info">Recommended: {probe.recommended_action}</Notice>
    </Card>
  );
}

function PayloadHexView({ hex }: { hex: string }) {
  const byteCount = hex.split(" ").filter(Boolean).length;

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Last HID feature report</p>
          <h2>{byteCount} bytes</h2>
        </div>
      </div>
      <div className="payload-preview">
        <span>Raw bytes</span>
        <code>{hex}</code>
      </div>
    </Card>
  );
}

function PayloadPreviewView({ preview }: { preview: HidPayloadPreview }) {
  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Last HID feature report</p>
          <h2>
            {preview.decoded_effect} · {preview.length} bytes
          </h2>
        </div>
      </div>
      <div className="payload-preview">
        <span>Raw bytes</span>
        <code>{preview.hex}</code>
      </div>
      <div className="diagnostics-table">
        <div>
          <span>Header</span>
          <strong>{preview.header_bytes.join(" ")}</strong>
        </div>
        <div>
          <span>Effect byte</span>
          <strong>
            {preview.effect_byte} ({preview.decoded_effect})
          </strong>
        </div>
        <div>
          <span>Speed byte</span>
          <strong>{preview.speed_byte}</strong>
        </div>
        <div>
          <span>Brightness byte</span>
          <strong>{preview.brightness_byte}</strong>
        </div>
        <div>
          <span>Direction bytes</span>
          <strong>{preview.direction_bytes.join(" ")}</strong>
        </div>
      </div>
      <div className="payload-preview">
        <span>Zone RGB bytes</span>
        <code>{preview.zone_bytes.join("  |  ")}</code>
      </div>
    </Card>
  );
}

function HidDevicesTable({ devices }: { devices: HidDeviceSummary[] }) {
  if (devices.length === 0) {
    return <Notice tone="info">No ITE 0x048d HID devices were reported.</Notice>;
  }

  return (
    <div className="hid-table">
      <div className="hid-table__header">
        <span>Vendor</span>
        <span>Product</span>
        <span>Name</span>
        <span>Status</span>
      </div>
      {devices.map((device) => (
        <div
          className="hid-table__row"
          key={`${device.vendor_id}-${device.product_id}-${device.path ?? ""}`}
        >
          <code>{device.vendor_id}</code>
          <code>{device.product_id}</code>
          <span title={device.path ?? undefined}>
            {device.product ?? device.manufacturer ?? "Unknown ITE HID device"}
          </span>
          <StatusBadge
            label={
              device.supported_for_writes
                ? "Enabled"
                : device.known
                  ? "Recognized only"
                  : "Unknown"
            }
            tone={device.supported_for_writes ? "ok" : "warn"}
          />
        </div>
      ))}
    </div>
  );
}
