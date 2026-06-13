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

export function rgbToCss(color: RgbColor | null): string {
  if (!color) {
    return "rgb(0 0 0)";
  }

  return `rgb(${color.r} ${color.g} ${color.b})`;
}
