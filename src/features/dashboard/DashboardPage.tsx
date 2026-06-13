import type { AppInfo } from "../../domain/app";
import type { DeviceInfo } from "../../domain/device";
import type { KeyboardState } from "../../domain/lighting";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { backendModeNotice } from "../../app/constants";
import { CurrentLightingCard } from "./CurrentLightingCard";
import { DeviceStatusCard } from "./DeviceStatusCard";

interface DashboardPageProps {
  appInfo: AppInfo | null;
  device: DeviceInfo | null;
  keyboardState: KeyboardState | null;
  loading: boolean;
  onApplyCurrent: () => void;
  onRefresh: () => void;
  onTurnOff: () => void;
}

export function DashboardPage({
  appInfo,
  device,
  keyboardState,
  loading,
  onApplyCurrent,
  onRefresh,
  onTurnOff,
}: DashboardPageProps) {
  return (
    <section className="page-stack">
      <div className="page-heading">
        <div>
          <p className="eyebrow">{appInfo?.backend_mode ?? "mock"} backend</p>
          <h1>{appInfo?.name ?? "LegionGlow"}</h1>
        </div>
        <div className="action-row">
          <Button disabled={loading} onClick={onRefresh}>
            Refresh
          </Button>
          <Button
            disabled={loading || !keyboardState}
            variant="primary"
            onClick={onApplyCurrent}
          >
            Apply mode
          </Button>
          <Button disabled={loading} variant="danger" onClick={onTurnOff}>
            Turn off
          </Button>
        </div>
      </div>

      <Notice tone={appInfo?.real_hardware_writes_enabled ? "warning" : "info"}>
        {backendModeNotice(appInfo?.backend_mode)}
      </Notice>

      {appInfo?.configuration_warnings.map((warning) => (
        <Notice key={warning} tone="warning">
          {warning}
        </Notice>
      ))}

      <Card className="backend-strip">
        <div>
          <span>Backend mode</span>
          <strong>{appInfo?.backend_mode ?? "mock"}</strong>
        </div>
        <div>
          <span>Backend write mode</span>
          <StatusBadge
            label={
              appInfo?.real_hardware_writes_enabled
                ? "Real backend selected"
                : "No hardware writes"
            }
            tone={appInfo?.real_hardware_writes_enabled ? "danger" : "ok"}
          />
        </div>
        <div>
          <span>Capabilities</span>
          <strong>
            {device?.capabilities.supports_zones
              ? `${device.capabilities.zone_count} zones`
              : "Unavailable"}
          </strong>
        </div>
      </Card>

      <div className="dashboard-grid">
        <DeviceStatusCard device={device} loading={loading} />
        <CurrentLightingCard state={keyboardState} />
      </div>
    </section>
  );
}
