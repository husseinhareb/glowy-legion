import type { KeyboardState } from "../domain/lighting";
import type { LightingProfile } from "../domain/profile";
import { invokeCommand } from "./tauriClient";

export function listBuiltinProfiles(): Promise<LightingProfile[]> {
  return invokeCommand<LightingProfile[]>("list_builtin_profiles");
}

export function applyProfile(profileId: string): Promise<KeyboardState> {
  return invokeCommand<KeyboardState>("apply_profile", { profileId });
}
