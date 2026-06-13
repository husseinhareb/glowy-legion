import { Button } from "../../shared/components/Button";
import { Notice } from "../../shared/components/Notice";

export interface ConfirmationContent {
  title: string;
  intro: string;
  lines: string[];
  confirmLabel: string;
  tone: "primary" | "danger";
}

interface PermissionConfirmationDialogProps {
  content: ConfirmationContent | null;
  password: string;
  busy: boolean;
  error: string | null;
  onPasswordChange: (value: string) => void;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * Confirmation shown before any privileged install/reload/remove. It explains
 * exactly what changes and collects the system password, which is sent to the
 * backend and piped to `sudo -S` over stdin. The password is used once and is
 * never stored or logged.
 */
export function PermissionConfirmationDialog({
  content,
  password,
  busy,
  error,
  onPasswordChange,
  onConfirm,
  onCancel,
}: PermissionConfirmationDialogProps) {
  if (!content) {
    return null;
  }

  const canConfirm = !busy && password.length > 0;

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal-card">
        <h2>{content.title}</h2>
        <p>{content.intro}</p>
        <ul className="modal-list">
          {content.lines.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>

        <label className="field">
          <span>System password (sudo)</span>
          <input
            autoFocus
            disabled={busy}
            placeholder="Your account password"
            type="password"
            value={password}
            onChange={(event) => onPasswordChange(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && canConfirm) {
                onConfirm();
              }
            }}
          />
        </label>
        <Notice tone="info">
          Your password is sent to the backend and piped to <code>sudo</code> to
          run this single command. It is used once and is not stored or logged.
        </Notice>

        {error && <Notice tone="error">{error}</Notice>}

        <div className="action-row">
          <Button
            disabled={!canConfirm}
            variant={content.tone}
            onClick={onConfirm}
          >
            {content.confirmLabel}
          </Button>
          <Button disabled={busy} onClick={onCancel}>
            Cancel
          </Button>
        </div>
      </div>
    </div>
  );
}
