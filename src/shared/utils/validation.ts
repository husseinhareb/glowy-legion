import type { DeviceCapabilities } from "../../domain/device";
import type { KeyboardState } from "../../domain/lighting";
import {
  effectUsesDirection,
  effectUsesSpeed,
  isDirectionSupportedForEffect,
  isEffectSupported,
} from "../../domain/lighting";

export function canApplyState(
  state: KeyboardState,
  capabilities: DeviceCapabilities | null,
): boolean {
  if (!capabilities) {
    return false;
  }

  if (!isEffectSupported(state.effect, capabilities)) {
    return false;
  }

  if (state.brightness > 0 && !capabilities.supports_brightness) {
    return false;
  }

  if (effectUsesSpeed(state.effect) && !capabilities.supports_speed) {
    return false;
  }

  if (effectUsesDirection(state.effect) && !capabilities.supports_direction) {
    return false;
  }

  if (!isDirectionSupportedForEffect(state.effect, state.direction)) {
    return false;
  }

  // Secondary color and per-zone colors are optional decoration: the backend
  // discards them when unsupported rather than rejecting, so they do not block
  // applying a state here either.

  return state.brightness >= 0 && state.brightness <= 100 && state.speed >= 0 && state.speed <= 100;
}
