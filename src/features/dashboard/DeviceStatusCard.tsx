import type { DeviceInfo } from "../../domain/device";
import { Card } from "../../shared/components/Card";
import { LoadingState } from "../../shared/components/LoadingState";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { formatDeviceFamily } from "../../shared/utils/formatting";

interface DeviceStatusCardProps {
  device: DeviceInfo | null;
  loading: boolean;
}

export function DeviceStatusCard({ device, loading }: DeviceStatusCardProps) {
  if (loading && !device) {
    return (
      <Card>
        <LoadingState label="Detecting keyboard device" />
      </Card>
    );
  }

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Detected device</p>
          <h2>{device?.product_name ?? "No device detected"}</h2>
        </div>
        <StatusBadge
          label={deviceStatusLabel(device)}
          tone={device?.supported ? "ok" : "danger"}
        />
      </div>
      <dl className="detail-grid">
        <div>
          <dt>Vendor</dt>
          <dd>{device?.vendor ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Family</dt>
          <dd>{device ? formatDeviceFamily(device.family) : "Unknown"}</dd>
        </div>
        <div>
          <dt>Backend</dt>
          <dd>{device?.backend ?? "mock"}</dd>
        </div>
        <div>
          <dt>Zones</dt>
          <dd>{device?.capabilities.zone_count ?? 0}</dd>
        </div>
      </dl>
    </Card>
  );
}

function deviceStatusLabel(device: DeviceInfo | null): string {
  if (!device) {
    return "Unknown";
  }

  if (!device.supported) {
    return "Unsupported";
  }

  if (device.backend === "lenovo-hid-dry-run") {
    return "Dry-run capable";
  }

  if (device.backend === "lenovo-hid") {
    return "Write capable";
  }

  return "Mock device";
}
