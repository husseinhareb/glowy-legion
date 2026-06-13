import type { HidAccessProbe, UdevRulePreview } from "../domain/diagnostics";
import type { PermissionSetupResult } from "../domain/permissions";
import { invokeCommand } from "./tauriClient";

export function previewUdevRule(): Promise<UdevRulePreview> {
  return invokeCommand<UdevRulePreview>("preview_udev_rule");
}

export function probeHidAccess(): Promise<HidAccessProbe> {
  return invokeCommand<HidAccessProbe>("probe_hid_access");
}

export function installUdevRuleWithSystemAuth(
  password: string,
): Promise<PermissionSetupResult> {
  return invokeCommand<PermissionSetupResult>(
    "install_udev_rule_with_system_auth",
    { password },
  );
}

export function reloadUdevRulesWithSystemAuth(
  password: string,
): Promise<PermissionSetupResult> {
  return invokeCommand<PermissionSetupResult>(
    "reload_udev_rules_with_system_auth",
    { password },
  );
}

export function removeUdevRuleWithSystemAuth(
  password: string,
): Promise<PermissionSetupResult> {
  return invokeCommand<PermissionSetupResult>(
    "remove_udev_rule_with_system_auth",
    { password },
  );
}
