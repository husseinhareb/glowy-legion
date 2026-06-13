import type { HidAccessProbe } from "../../domain/diagnostics";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";

interface HidAccessProbeCardProps {
  probe: HidAccessProbe | null;
  loading: boolean;
  /** Number of interfaces classified safe for an RGB probe. */
  eligibleInterfaceCount: number;
  /** LEGIONGLOW_DISABLE_HID blocks all active HID access. */
  hidAccessDisabled: boolean;
  onProbe: () => void;
}

export function HidAccessProbeCard({
  probe,
  loading,
  eligibleInterfaceCount,
  hidAccessDisabled,
  onProbe,
}: HidAccessProbeCardProps) {
  const canOpen = probe?.can_open ?? false;
  // The probe never runs automatically. The button stays disabled unless
  // exactly one safe RGB-control interface was identified.
  const probeAllowed =
    !hidAccessDisabled && eligibleInterfaceCount === 1 && !loading;

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">HID access (manual probe)</p>
          <h3>
            {probe ? `${probe.vendor_id}:${probe.product_id}` : "Not probed yet"}
          </h3>
        </div>
        <div className="action-row">
          <StatusBadge
            label={
              !probe
                ? "Not probed"
                : canOpen
                  ? "Access OK"
                  : "Blocked"
            }
            tone={!probe ? "warn" : canOpen ? "ok" : "danger"}
          />
          <Button disabled={!probeAllowed} onClick={onProbe}>
            Probe HID access
          </Button>
        </div>
      </div>

      {hidAccessDisabled ? (
        <Notice tone="warning">HID access disabled by safety flag.</Notice>
      ) : eligibleInterfaceCount === 1 ? (
        <Notice tone="warning">
          This will briefly open the selected HID interface and, for a
          LampArray lighting interface, read its read-only attributes (lamp
          count). It will not send RGB data or change any lighting state.
        </Notice>
      ) : (
        <Notice tone="warning">
          No safe RGB-control HID interface was identified. LegionGlow will not
          open this device.
        </Notice>
      )}

      {probe && (
        <div className="diagnostics-table">
          <div>
            <span>Device</span>
            <strong>{probe.product ?? probe.label}</strong>
          </div>
          <div>
            <span>Detected</span>
            <strong>{probe.path_available ? "true" : "false"}</strong>
          </div>
          <div>
            <span>Can open</span>
            <strong>{canOpen ? "yes" : "no"}</strong>
          </div>
          <div>
            <span>Failure kind</span>
            <strong>{probe.failure_kind ?? "None"}</strong>
          </div>
        </div>
      )}

      {probe?.lamp_array_attributes && (
        <div className="diagnostics-table">
          <div>
            <span>LampArray lamps</span>
            <strong>{probe.lamp_array_attributes.lamp_count}</strong>
          </div>
          <div>
            <span>LampArray kind</span>
            <strong>{probe.lamp_array_attributes.kind_label}</strong>
          </div>
          <div>
            <span>Min update interval</span>
            <strong>
              {(
                probe.lamp_array_attributes.min_update_interval_microseconds /
                1000
              ).toFixed(1)}{" "}
              ms
            </strong>
          </div>
        </div>
      )}

      {probe && (
        <Notice tone={canOpen ? "success" : "warning"}>
          {probe.user_message}
        </Notice>
      )}
      {probe && !canOpen && (
        <Notice tone="info">Next step: {probe.recommended_action}</Notice>
      )}
    </Card>
  );
}
