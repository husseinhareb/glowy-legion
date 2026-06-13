import { Button } from "../../shared/components/Button";

interface PermissionActionButtonsProps {
  busy: boolean;
  available: boolean;
  onInstall: () => void;
  onReload: () => void;
  onRemove: () => void;
  onCopyDiagnostics: () => void;
}

/**
 * Privileged actions. Each opens a confirmation dialog that collects the system
 * password, which the backend pipes to `sudo -S` over stdin to run a single
 * fixed command. The password is used once and never stored or logged.
 */
export function PermissionActionButtons({
  busy,
  available,
  onInstall,
  onReload,
  onRemove,
  onCopyDiagnostics,
}: PermissionActionButtonsProps) {
  return (
    <div className="action-row action-row--wrap">
      <Button
        disabled={busy || !available}
        variant="primary"
        onClick={onInstall}
      >
        Install udev rule with system authentication
      </Button>
      <Button disabled={busy} onClick={onReload}>
        Reload udev rules with system authentication
      </Button>
      <Button disabled={busy} variant="danger" onClick={onRemove}>
        Remove LegionGlow udev rule
      </Button>
      <Button disabled={busy} onClick={onCopyDiagnostics}>
        Copy diagnostics JSON
      </Button>
    </div>
  );
}
