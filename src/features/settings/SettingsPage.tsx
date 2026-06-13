import { useCallback, useEffect, useState } from "react";

import { runDiagnostics } from "../../api/diagnosticsApi";
import { sendSafeTestPayload, setKeyboardState } from "../../api/lightingApi";
import type { AppInfo } from "../../domain/app";
import type { DeviceInfo } from "../../domain/device";
import type { DiagnosticsReport, HidAccessProbe } from "../../domain/diagnostics";
import { createDefaultKeyboardState } from "../../domain/lighting";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { useAsync } from "../../shared/hooks/useAsync";
import { PermissionSetupPanel } from "../permissions/PermissionSetupPanel";
import { BackendModeNotice } from "./BackendModeNotice";
import { StageCard, type StageTone } from "./StageCard";

interface SettingsPageProps {
  appInfo: AppInfo | null;
  device: DeviceInfo | null;
}

const REAL_MODE_COMMAND = "npm run tauri:real dev";

export function SettingsPage({ appInfo, device }: SettingsPageProps) {
  const diagnostics = useAsync<DiagnosticsReport>();
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // Result of the manual "Probe HID access" action. Diagnostics never probes.
  const [probe, setProbe] = useState<HidAccessProbe | null>(null);

  const refresh = useCallback(() => {
    diagnostics.run(runDiagnostics).catch(() => undefined);
  }, [diagnostics.run]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const report = diagnostics.value;
  const candidate = report?.known_supported_lenovo_rgb_device ?? null;
  const canOpen = probe?.can_open ?? false;
  const realWritesEnabled = report?.real_hardware_writes_enabled ?? false;
  const hasPayloadPreview = (report?.last_payload_hex ?? null) !== null;

  const runAction = async (task: () => Promise<unknown>, message: string) => {
    setBusy(true);
    setActionStatus(null);
    setActionError(null);
    try {
      await task();
      setActionStatus(message);
      refresh();
    } catch (error) {
      setActionError(
        error instanceof Error ? error.message : String(error ?? "Unknown error"),
      );
    } finally {
      setBusy(false);
    }
  };

  const deviceStatus: StageTone = device?.supported ? "ok" : "warn";
  const permissionStatus: StageTone = !probe
    ? "warn"
    : canOpen
      ? "ok"
      : "danger";
  const permissionLabel = !probe ? "Unknown" : canOpen ? "Fixed" : "Blocked";
  const dryRunStatus: StageTone = hasPayloadPreview ? "ok" : "warn";

  return (
    <section className="page-stack">
      <div className="page-heading">
        <div>
          <p className="eyebrow">Settings</p>
          <h1>Staged setup</h1>
        </div>
      </div>

      <BackendModeNotice backendMode={appInfo?.backend_mode} />

      {actionStatus && <Notice tone="success">{actionStatus}</Notice>}
      {actionError && <Notice tone="error">{actionError}</Notice>}

      <StageCard
        description={
          device
            ? `${device.product_name} (${report?.dmi_product_name ?? "unknown DMI"})`
            : "No device detected yet."
        }
        index={1}
        statusLabel={device?.supported ? "OK" : "Detected"}
        statusTone={deviceStatus}
        title="Detect device"
      >
        <div className="diagnostics-table">
          <div>
            <span>HID candidate</span>
            <strong>
              {candidate
                ? `${candidate.vendor_id}:${candidate.product_id}`
                : "Pending"}
            </strong>
          </div>
          <div>
            <span>Laptop</span>
            <strong>
              {report?.dmi_product_version ?? device?.product_name ?? "Unknown"}
            </strong>
          </div>
        </div>
      </StageCard>

      <StageCard
        description="Install the udev rule with system authentication so LegionGlow can open the HID device without root."
        index={2}
        statusLabel={permissionLabel}
        statusTone={permissionStatus}
        title="Fix permissions"
      >
        <PermissionSetupPanel
          candidateLabel={
            candidate ? `${candidate.vendor_id}:${candidate.product_id}` : null
          }
          diagnosticsJson={report ? JSON.stringify(report, null, 2) : null}
          eligibleInterfaceCount={report?.eligible_rgb_interface_count ?? 0}
          hidAccessDisabled={report?.hid_access_disabled_by_safety_flag ?? false}
          interfaces={report?.hid_interfaces ?? []}
          onProbeResult={setProbe}
          onRefreshDiagnostics={refresh}
        />
      </StageCard>

      <StageCard
        description="Generate payload bytes in dry-run mode to confirm the protocol encodes as expected. No feature reports are sent."
        index={3}
        statusLabel={hasPayloadPreview ? "OK" : "Pending"}
        statusTone={dryRunStatus}
        title="Dry-run validation"
      >
        <div className="action-row">
          <Button
            disabled={busy}
            onClick={() =>
              runAction(
                () => setKeyboardState(createDefaultKeyboardState()),
                "Payload bytes generated in dry-run mode.",
              )
            }
          >
            Generate payload bytes
          </Button>
        </div>
      </StageCard>

      <StageCard
        description="Real writes are never enabled automatically. Built-in devices require the real backend; experimental product IDs also require an explicit override."
        index={4}
        statusLabel={realWritesEnabled ? "Active" : "Locked"}
        statusTone={realWritesEnabled ? "danger" : "warn"}
        title="Experimental real mode"
      >
        <Notice tone="warning">
          This can send real HID feature reports to hardware. Use only after
          dry-run validation and permission checks.
        </Notice>
        <div className="payload-preview">
          <span>Run LegionGlow in real mode manually</span>
          <code>{REAL_MODE_COMMAND}</code>
        </div>
        <p className="prereq-line">
          <span>can_open: true is only one requirement</span>
          <span aria-hidden> · </span>
          <span>real backend + write-enabled product also required</span>
          <span aria-hidden> · </span>
          <span>confirm with safe test payload first</span>
        </p>
      </StageCard>

      <StageCard
        description="Lowest-risk real write: static dim blue at minimum brightness. Disabled unless real writes are enabled."
        index={5}
        statusLabel={realWritesEnabled ? "Ready" : "Disabled"}
        statusTone={realWritesEnabled ? "danger" : "warn"}
        title="Safe first write"
      >
        <Notice tone="warning">
          This sends a real HID feature report. Use only after dry-run and
          permissions are verified.
        </Notice>
        <div className="action-row">
          <Button
            disabled={busy || !realWritesEnabled}
            variant="danger"
            onClick={() =>
              runAction(() => sendSafeTestPayload(), "Safe test payload sent.")
            }
          >
            Send safe test payload
          </Button>
        </div>
      </StageCard>

      <div className="settings-grid">
        <Card>
          <div className="card__header">
            <div>
              <p className="eyebrow">Runtime</p>
              <h2>{appInfo?.backend_mode ?? "mock"}</h2>
            </div>
            <StatusBadge
              label={
                appInfo?.real_hardware_writes_enabled
                  ? "Real backend selected"
                  : appInfo?.backend_mode === "lenovo-hid-dry-run"
                    ? "Dry-run active"
                    : "Mock active"
              }
              tone={appInfo?.real_hardware_writes_enabled ? "danger" : "ok"}
            />
          </div>
          {appInfo?.configuration_warnings.map((warning) => (
            <Notice key={warning} tone="warning">
              {warning}
            </Notice>
          ))}
        </Card>

        <Card>
          <div className="card__header">
            <div>
              <p className="eyebrow">Device capability model</p>
              <h2>{device?.product_name ?? "No device loaded"}</h2>
            </div>
          </div>
          <dl className="capability-list">
            {device &&
              Object.entries(device.capabilities).map(([key, value]) => (
                <div key={key}>
                  <dt>{key.replace(/_/g, " ")}</dt>
                  <dd>{String(value)}</dd>
                </div>
              ))}
          </dl>
        </Card>
      </div>
    </section>
  );
}
