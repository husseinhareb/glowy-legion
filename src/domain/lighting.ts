import type { DeviceCapabilities } from "./device";

export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

/**
 * Per-zone color override. Zone indices are zero-based: zone 0 is the leftmost
 * zone, the highest valid index is `zone_count - 1` (so 0..=3 on a 4-zone board).
 */
export interface ZoneColor {
  zone_index: number;
  color: RgbColor;
}

export type LightingEffect =
  | "Static"
  | "Breathing"
  | "Wave"
  | "Reactive"
  | "Rainbow"
  | "Off";

export type EffectDirection =
  | "LeftToRight"
  | "RightToLeft"
  | "TopToBottom"
  | "BottomToTop";

export interface KeyboardState {
  effect: LightingEffect;
  primary_color: RgbColor;
  secondary_color: RgbColor | null;
  brightness: number;
  speed: number;
  direction: EffectDirection;
  enabled: boolean;
  /** Optional per-zone colors. Null means "use primary color for all zones". */
  zone_colors: ZoneColor[] | null;
}

/** Effects whose per-zone colors are honored by the protocol. */
export function effectSupportsZoneColors(effect: LightingEffect): boolean {
  return effect === "Static" || effect === "Breathing";
}

/** Build a default 4-zone palette seeded from a single color. */
export function createDefaultZoneColors(
  zoneCount: number,
  seed: RgbColor,
): ZoneColor[] {
  return Array.from({ length: zoneCount }, (_unused, index) => ({
    zone_index: index,
    color: { ...seed },
  }));
}

export const LIGHTING_EFFECTS: LightingEffect[] = [
  "Static",
  "Breathing",
  "Wave",
  "Reactive",
  "Rainbow",
  "Off",
];

export const EFFECT_DIRECTIONS: EffectDirection[] = [
  "LeftToRight",
  "RightToLeft",
  "TopToBottom",
  "BottomToTop",
];

export const EFFECT_LABELS: Record<LightingEffect, string> = {
  Static: "Static",
  Breathing: "Breathing",
  Wave: "Wave",
  Reactive: "Reactive",
  Rainbow: "Rainbow",
  Off: "Off",
};

export const DIRECTION_LABELS: Record<EffectDirection, string> = {
  LeftToRight: "Left to right",
  RightToLeft: "Right to left",
  TopToBottom: "Top to bottom",
  BottomToTop: "Bottom to top",
};

export function isEffectSupported(
  effect: LightingEffect,
  capabilities: DeviceCapabilities | null,
): boolean {
  if (!capabilities) {
    return false;
  }

  switch (effect) {
    case "Static":
      return capabilities.supports_static;
    case "Breathing":
      return capabilities.supports_breathing;
    case "Wave":
      return capabilities.supports_wave;
    case "Reactive":
      return capabilities.supports_reactive;
    case "Rainbow":
      return capabilities.supports_rainbow;
    case "Off":
      return true;
  }
}

export function effectUsesSpeed(effect: LightingEffect): boolean {
  return ["Breathing", "Wave", "Reactive", "Rainbow"].includes(effect);
}

export function effectUsesDirection(effect: LightingEffect): boolean {
  return ["Wave", "Rainbow"].includes(effect);
}

export function isDirectionSupportedForEffect(
  effect: LightingEffect,
  direction: EffectDirection,
): boolean {
  if (!effectUsesDirection(effect)) {
    return true;
  }

  return direction === "LeftToRight" || direction === "RightToLeft";
}

export function createDefaultKeyboardState(): KeyboardState {
  return {
    effect: "Static",
    primary_color: { r: 0, g: 148, b: 255 },
    secondary_color: { r: 0, g: 255, b: 210 },
    brightness: 70,
    speed: 45,
    direction: "LeftToRight",
    enabled: true,
    zone_colors: null,
  };
}
