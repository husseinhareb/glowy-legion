import type { HidAccessProbe, UdevRulePreview } from "./diagnostics";

export type { HidAccessProbe, UdevRulePreview };

/**
 * Result of a privileged permission-setup action. Never contains a password or
 * any authentication secret — the password is piped to `sudo -S` over stdin for
 * a single command, and only the resulting process output is captured here.
 */
export interface PermissionSetupResult {
  success: boolean;
  action: string;
  message: string;
  stdout: string | null;
  stderr: string | null;
  requires_reconnect: boolean;
  next_steps: string[];
  warnings: string[];
}
