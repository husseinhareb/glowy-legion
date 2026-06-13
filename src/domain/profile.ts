import type { KeyboardState } from "./lighting";

export interface ProfileCompatibility {
  supported: boolean;
  reasons: string[];
}

export interface LightingProfile {
  id: string;
  name: string;
  description: string;
  state: KeyboardState;
  compatibility: ProfileCompatibility;
}
