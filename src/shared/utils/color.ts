import type { RgbColor } from "../../domain/lighting";

export function rgbToHex(color: RgbColor): string {
  const channelToHex = (value: number) =>
    Math.max(0, Math.min(255, value)).toString(16).padStart(2, "0");

  return `#${channelToHex(color.r)}${channelToHex(color.g)}${channelToHex(
    color.b,
  )}`;
}

export function hexToRgb(hex: string): RgbColor {
  const normalized = hex.replace("#", "");
  const value = Number.parseInt(normalized, 16);

  return {
    r: (value >> 16) & 255,
    g: (value >> 8) & 255,
    b: value & 255,
  };
}

/** Linearly interpolate between two colors. `t` is clamped to [0, 1]. */
export function lerpRgb(a: RgbColor, b: RgbColor, t: number): RgbColor {
  const clamped = Math.max(0, Math.min(1, t));
  const channel = (from: number, to: number) =>
    Math.round(from + (to - from) * clamped);

  return {
    r: channel(a.r, b.r),
    g: channel(a.g, b.g),
    b: channel(a.b, b.b),
  };
}

/** Convert HSV (each in [0, 1]) to RGB. Used for the rainbow fill. */
export function hsvToRgb(h: number, s: number, v: number): RgbColor {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);

  const [r, g, b] = [
    [v, t, p],
    [q, v, p],
    [p, v, t],
    [p, q, v],
    [t, p, v],
    [v, p, q],
  ][((i % 6) + 6) % 6];

  return {
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255),
  };
}

export function rgbToCss(color: RgbColor | null): string {
  if (!color) {
    return "rgb(0 0 0)";
  }

  return `rgb(${color.r} ${color.g} ${color.b})`;
}
