import { useEffect, useState } from "react";

import {
  installUdevRuleWithSystemAuth,
  previewUdevRule,
  probeHidAccess,
  reloadUdevRulesWithSystemAuth,
  removeUdevRuleWithSystemAuth,
} from "../../api/permissionApi";
import type {
  HidAccessProbe,
  HidInterfaceSummary,
  UdevRulePreview,
} from "../../domain/diagnostics";
import type { PermissionSetupResult } from "../../domain/permissions";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { useAsync } from "../../shared/hooks/useAsync";
import { HidInterfacesTable } from "../diagnostics/HidInterfacesTable";
import { HidAccessProbeCard } from "./HidAccessProbeCard";
import { PermissionActionButtons } from "./PermissionActionButtons";
import {
  PermissionConfirmationDialog,
  type ConfirmationContent,
} from "./PermissionConfirmationDialog";
import { PermissionSetupResultCard } from "./PermissionSetupResultCard";
import { UdevRulePreviewCard } from "./UdevRulePreviewCard";

interface PermissionSetupPanelProps {
  candidateLabel: string | null;
  interfaces: HidInterfaceSummary[];
  eligibleInterfaceCount: number;
  hidAccessDisabled: boolean;
  diagnosticsJson: string | null;
  onRefreshDiagnostics: () => void;
  onProbeResult?: (probe: HidAccessProbe) => void;
}

type PendingAction = "install" | "reload" | "remove";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error ?? "Unknown error");
}

export function PermissionSetupPanel({
  candidateLabel,
  interfaces,
  eligibleInterfaceCount,
  hidAccessDisabled,
  diagnosticsJson,
  onRefreshDiagnostics,
  onProbeResult,
}: PermissionSetupPanelProps) {
  const preview = useAsync<UdevRulePreview>();
  const [probe, setProbe] = useState<HidAccessProbe | null>(null);
  const [result, setResult] = useState<PermissionSetupResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<PendingAction | null>(null);
  const [password, setPassword] = useState("");
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    preview.run(previewUdevRule).catch(() => undefined);
  }, [preview.run]);

  const previewValue = preview.value;
  const filename = previewValue?.filename ?? "99-legionglow-lenovo-rgb.rules";
  const rulePath = `/etc/udev/rules.d/${filename}`;

  const copy = async (text: string, message: string) => {
    setError(null);
    await navigator.clipboard.writeText(text);
    setStatus(message);
  };

  const openDialog = (action: PendingAction) => {
    setPassword("");
    setPasswordError(null);
    setPending(action);
  };

  const closeDialog = () => {
    setPending(null);
    setPassword("");
    setPasswordError(null);
  };

  const confirmAction = async () => {
    if (!pending) {
      return;
    }
    const call =
      pending === "install"
        ? () => installUdevRuleWithSystemAuth(password)
        : pending === "remove"
          ? () => removeUdevRuleWithSystemAuth(password)
          : () => reloadUdevRulesWithSystemAuth(password);

    setBusy(true);
    setStatus(null);
    setError(null);
    try {
      const outcome = await call();
      const wrongPassword =
        !outcome.success && /incorrect password/i.test(outcome.message);
      if (wrongPassword) {
        // Keep the dialog open so the user can retry.
        setPasswordError(outcome.message);
        setPassword("");
        return;
      }

      setResult(outcome);
      // Deliberately no automatic HID re-probe here: HID interfaces are only
      // opened when the user explicitly clicks "Probe HID access".
      onRefreshDiagnostics();
      closeDialog();
    } catch (caught) {
      setPasswordError(errorMessage(caught));
      setPassword("");
    } finally {
      setBusy(false);
    }
  };

  const runProbe = async () => {
    setBusy(true);
    setError(null);
    try {
      const outcome = await probeHidAccess();
      setProbe(outcome);
      onProbeResult?.(outcome);
      onRefreshDiagnostics();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  };

  const confirmContent: ConfirmationContent | null =
    pending === "install"
      ? {
          title: "Install udev rule",
          intro: "LegionGlow will install one udev rule using your system password.",
          lines: [
            `Rule: ${previewValue?.rule ?? "(generated for the detected device)"}`,
            `Destination: ${rulePath}`,
            "What it changes: grants your local session access to this single USB device.",
            "This installs the rule, reloads udev, and re-triggers the device in one step.",
            "If access is still blocked afterward, log out and back in (or reboot) — an internal keyboard cannot be replugged.",
            "Installing permissions does NOT mean the device is safe to open, and it does NOT enable real writes.",
          ],
          confirmLabel: "Authenticate & install",
          tone: "primary",
        }
      : pending === "reload"
        ? {
            title: "Reload udev rules",
            intro: "LegionGlow will reload udev rules using your system password.",
            lines: [
              "Runs: udevadm control --reload-rules, then udevadm trigger.",
              "This re-applies installed rules without a reboot.",
            ],
            confirmLabel: "Authenticate & reload",
            tone: "primary",
          }
        : pending === "remove"
          ? {
              title: "Remove LegionGlow udev rule",
              intro:
                "LegionGlow will remove only its own managed rule using your system password.",
              lines: [
                `File to remove: ${rulePath}`,
                "Only the LegionGlow-managed rule is removed; other rules are untouched.",
                "This may disable non-root HID access again.",
              ],
              confirmLabel: "Authenticate & remove",
              tone: "danger",
            }
          : null;

  return (
    <div className="permission-setup">
      <Card>
        <div className="card__header">
          <div>
            <p className="eyebrow">Permission Setup</p>
            <h2>Fix HID permissions</h2>
          </div>
        </div>
        {hidAccessDisabled && (
          <Notice tone="warning">HID access disabled by safety flag.</Notice>
        )}
        <Notice tone="info">
          Privileged actions run through <code>sudo</code>. You enter your system
          password in the dialog; it is sent to the backend and piped to{" "}
          <code>sudo</code> once to run that single command. It is not stored or
          logged, and LegionGlow never runs the whole GUI as root. Installing
          permissions does not imply the device is safe to open.
        </Notice>
        <div className="diagnostics-table">
          <div>
            <span>Current device</span>
            <strong>{candidateLabel ?? "Not detected"}</strong>
          </div>
          <div>
            <span>Safe RGB-control interfaces</span>
            <strong>{eligibleInterfaceCount}</strong>
          </div>
        </div>
        {status && <Notice tone="success">{status}</Notice>}
        {error && <Notice tone="error">{error}</Notice>}
        <PermissionActionButtons
          available={previewValue?.available ?? false}
          busy={busy}
          onInstall={() => openDialog("install")}
          onReload={() => openDialog("reload")}
          onRemove={() => openDialog("remove")}
          onCopyDiagnostics={() =>
            diagnosticsJson
              ? copy(diagnosticsJson, "Diagnostics JSON copied to clipboard.")
              : setError("Diagnostics are not loaded yet.")
          }
        />
      </Card>

      <Card>
        <div className="card__header">
          <div>
            <p className="eyebrow">Detected HID interfaces</p>
            <h3>Interface safety</h3>
          </div>
        </div>
        <HidInterfacesTable interfaces={interfaces} />
      </Card>

      <HidAccessProbeCard
        eligibleInterfaceCount={eligibleInterfaceCount}
        hidAccessDisabled={hidAccessDisabled}
        loading={busy}
        probe={probe}
        onProbe={runProbe}
      />

      <UdevRulePreviewCard
        error={preview.error}
        loading={preview.loading}
        preview={previewValue}
        onCopyManual={() =>
          previewValue &&
          copy(
            previewValue.install_commands.join("\n"),
            "Manual install commands copied to clipboard.",
          )
        }
        onCopyRule={() =>
          previewValue &&
          copy(previewValue.rule, "udev rule copied to clipboard.")
        }
        onPreview={() => preview.run(previewUdevRule).catch(() => undefined)}
      />

      <PermissionSetupResultCard result={result} />

      <PermissionConfirmationDialog
        busy={busy}
        content={confirmContent}
        error={passwordError}
        password={password}
        onCancel={closeDialog}
        onConfirm={confirmAction}
        onPasswordChange={setPassword}
      />
    </div>
  );
}
