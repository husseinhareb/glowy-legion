import type { DeviceFamily } from "../../domain/device";
import type { EffectDirection, LightingEffect } from "../../domain/lighting";
import { DIRECTION_LABELS, EFFECT_LABELS } from "../../domain/lighting";

export function formatEffect(effect: LightingEffect): string {
  return EFFECT_LABELS[effect];
}

export function formatDirection(direction: EffectDirection): string {
  return DIRECTION_LABELS[direction];
}

export function formatDeviceFamily(family: DeviceFamily): string {
  const labels: Record<DeviceFamily, string> = {
    LenovoLegion: "Lenovo Legion",
    LenovoLoq: "Lenovo LOQ",
    LenovoUnknown: "Unknown Lenovo laptop",
    Unsupported: "Unsupported device",
  };

  return labels[family];
}
