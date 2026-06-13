import type { CSSProperties } from "react";

import {
  effectSupportsZoneColors,
  type KeyboardState,
  type RgbColor,
} from "../../domain/lighting";
import { Card } from "../../shared/components/Card";
import { rgbToCss } from "../../shared/utils/color";

const BOARD_WIDTH = 1452;
const BOARD_HEIGHT = 474;

interface PreviewKey {
  id: string;
  label: string;
  subLabel?: string;
  x: number;
  y: number;
  w: number;
  h: number;
  align?: "left" | "center" | "right";
  compact?: boolean;
}

type KeyStyle = CSSProperties & {
  "--key-color": string;
  "--key-glow": string;
  "--legend-color": string;
};

const key = (
  id: string,
  label: string,
  x: number,
  y: number,
  w = 68,
  h = 68,
  subLabel?: string,
  align: PreviewKey["align"] = "left",
  compact = false,
): PreviewKey => ({ id, label, subLabel, x, y, w, h, align, compact });

const MARGIN = 12;
const TOP_Y = 12;
const FN_H = 40;
const FN_W = 72;
const NUM_Y = 60;
const Q_Y = 135;
const A_Y = 210;
const Z_Y = 285;
const MOD_Y = 360;
// Arrow cluster sits half a key lower than the bottom row, like the real deck.
const ARROW_Y = MOD_Y + 34;
const KEY_W = 68;
const KEY_H = 68;
const GAP = 7;
const PITCH = KEY_W + GAP;
const mainX = (index: number) => MARGIN + index * PITCH;
// 18 half-height keys spread edge to edge across the full board width.
const fnX = (index: number) => Math.round(MARGIN + index * 79.76);
const NP_X = [1147, 1222, 1297, 1372];

const FN_ROW: Array<[id: string, label: string, subLabel?: string]> = [
  ["esc", "Esc", "FnLock"],
  ["f1", "F1", "Mute"],
  ["f2", "F2", "Vol-"],
  ["f3", "F3", "Vol+"],
  ["f4", "F4", "Mic"],
  ["f5", "F5", "Dim"],
  ["f6", "F6", "Bright"],
  ["f7", "F7", "Display"],
  ["f8", "F8", "Air"],
  ["f9", "F9", "Star"],
  ["f10", "F10", "Cam"],
  ["f11", "F11", "Mode"],
  ["f12", "F12", "Calc"],
  ["insert", "Insert"],
  ["print", "PrtSc", "Snip"],
  ["delete", "Del"],
  ["home", "Home", "Play"],
  ["end", "End"],
];

const KEYS: PreviewKey[] = [
  ...FN_ROW.map(([id, label, subLabel], index) =>
    key(id, label, fnX(index), TOP_Y, FN_W, FN_H, subLabel, "center", true),
  ),

  key("grave", "~", mainX(0), NUM_Y, KEY_W, KEY_H, "`"),
  key("digit1", "!", mainX(1), NUM_Y, KEY_W, KEY_H, "1"),
  key("digit2", "@", mainX(2), NUM_Y, KEY_W, KEY_H, "2"),
  key("digit3", "#", mainX(3), NUM_Y, KEY_W, KEY_H, "3"),
  key("digit4", "$", mainX(4), NUM_Y, KEY_W, KEY_H, "4"),
  key("digit5", "%", mainX(5), NUM_Y, KEY_W, KEY_H, "5"),
  key("digit6", "^", mainX(6), NUM_Y, KEY_W, KEY_H, "6"),
  key("digit7", "&", mainX(7), NUM_Y, KEY_W, KEY_H, "7"),
  key("digit8", "*", mainX(8), NUM_Y, KEY_W, KEY_H, "8"),
  key("digit9", "(", mainX(9), NUM_Y, KEY_W, KEY_H, "9"),
  key("digit0", ")", mainX(10), NUM_Y, KEY_W, KEY_H, "0"),
  key("minus", "_", mainX(11), NUM_Y, KEY_W, KEY_H, "-"),
  key("equal", "+", mainX(12), NUM_Y, KEY_W, KEY_H, "="),
  key("backspace", "Backspace", mainX(13), NUM_Y, 143, KEY_H, undefined, "right"),
  key("numlock", "Num", NP_X[0], NUM_Y, KEY_W, KEY_H, "Lock", "left", true),
  key("numpad-divide", "/", NP_X[1], NUM_Y, KEY_W, KEY_H),
  key("numpad-multiply", "*", NP_X[2], NUM_Y, KEY_W, KEY_H),
  key("numpad-minus", "-", NP_X[3], NUM_Y, KEY_W, KEY_H),

  key("tab", "Tab", MARGIN, Q_Y, 102, KEY_H),
  ...["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"].map((label, index) =>
    key(`key-${label.toLowerCase()}`, label, 121 + index * PITCH, Q_Y),
  ),
  key("bracket-left", "{", 871, Q_Y, KEY_W, KEY_H, "["),
  key("bracket-right", "}", 946, Q_Y, KEY_W, KEY_H, "]"),
  key("backslash", "|", 1021, Q_Y, 109, KEY_H, "\\"),
  key("numpad7", "7", NP_X[0], Q_Y, KEY_W, KEY_H, "Home"),
  key("numpad8", "8", NP_X[1], Q_Y, KEY_W, KEY_H, "↑"),
  key("numpad9", "9", NP_X[2], Q_Y, KEY_W, KEY_H, "PgUp"),
  key("numpad-plus", "+", NP_X[3], Q_Y, KEY_W, 143),

  key("caps", "CapsLk", MARGIN, A_Y, 120, KEY_H),
  ...["A", "S", "D", "F", "G", "H", "J", "K", "L"].map((label, index) =>
    key(`key-${label.toLowerCase()}`, label, 139 + index * PITCH, A_Y),
  ),
  key("semicolon", ":", 814, A_Y, KEY_W, KEY_H, ";"),
  key("quote", '"', 889, A_Y, KEY_W, KEY_H, "'"),
  key("enter", "Enter", 964, A_Y, 166, KEY_H, undefined, "right"),
  key("numpad4", "4", NP_X[0], A_Y, KEY_W, KEY_H, "←"),
  key("numpad5", "5", NP_X[1], A_Y, KEY_W, KEY_H),
  key("numpad6", "6", NP_X[2], A_Y, KEY_W, KEY_H, "→"),

  key("shift-left", "Shift", MARGIN, Z_Y, 158, KEY_H),
  ...["Z", "X", "C", "V", "B", "N", "M"].map((label, index) =>
    key(`key-${label.toLowerCase()}`, label, 177 + index * PITCH, Z_Y),
  ),
  key("comma", "<", 702, Z_Y, KEY_W, KEY_H, ","),
  key("period", ">", 777, Z_Y, KEY_W, KEY_H, "."),
  key("slash", "?", 852, Z_Y, KEY_W, KEY_H, "/"),
  key("shift-right", "Shift", 927, Z_Y, 203, KEY_H, undefined, "right"),
  key("numpad1", "1", NP_X[0], Z_Y, KEY_W, KEY_H, "End"),
  key("numpad2", "2", NP_X[1], Z_Y, KEY_W, KEY_H, "↓"),
  key("numpad3", "3", NP_X[2], Z_Y, KEY_W, KEY_H, "PgDn"),
  key("numpad-enter", "Enter", NP_X[3], Z_Y, KEY_W, 143, undefined, "center", true),

  key("ctrl-left", "Ctrl", MARGIN, MOD_Y, 80, KEY_H),
  key("fn", "Fn", 99, MOD_Y, 72, KEY_H),
  key("meta", "⊞", 178, MOD_Y, 72, KEY_H, undefined, "center"),
  key("alt-left", "Alt", 257, MOD_Y, 72, KEY_H),
  key("space", "☼        ☼", 336, MOD_Y, 421, KEY_H, undefined, "center"),
  key("alt-right", "Alt", 764, MOD_Y, 72, KEY_H),
  key("ctrl-right", "Ctrl", 843, MOD_Y, 72, KEY_H),
  key("numpad0", "0", NP_X[0], MOD_Y, 143, KEY_H, "Ins"),
  key("numpad-decimal", ".", NP_X[2], MOD_Y, KEY_W, KEY_H, "Del"),

  key("arrow-left", "←", 922, ARROW_Y, KEY_W, KEY_H, undefined, "center"),
  key("arrow-up", "↑", 997, ARROW_Y, KEY_W, 31, undefined, "center", true),
  key("arrow-down", "↓", 997, ARROW_Y + 37, KEY_W, 31, undefined, "center", true),
  key("arrow-right", "→", 1072, ARROW_Y, KEY_W, KEY_H, undefined, "center"),
];

interface KeyboardPreviewProps {
  state: KeyboardState;
  /** Number of paintable segments (lamps). Required for `onPaintSegment`. */
  segmentCount?: number;
  /** When set, keys become clickable and paint their segment on click. */
  onPaintSegment?: (segmentIndex: number) => void;
}

export function KeyboardPreview({
  state,
  segmentCount,
  onPaintSegment,
}: KeyboardPreviewProps) {
  const active =
    state.enabled && state.effect !== "Off" && state.brightness > 0;
  const palette = resolvePreviewPalette(state);
  const editable = !!onPaintSegment && !!segmentCount && segmentCount > 0;

  return (
    <Card className="keyboard-preview-card">
      <div className="card__header">
        <div>
          <p className="eyebrow">Keyboard preview</p>
          <h2>LOQ 17IRX10 layout</h2>
        </div>
      </div>
      <div className="keyboard-preview" aria-label="Keyboard color preview">
        <div className="keyboard-preview__board">
          {KEYS.map((previewKey) => {
            const color = active
              ? palette[segmentForKey(previewKey, palette.length)]
              : { r: 72, g: 68, b: 94 };
            const glow = active
              ? Math.max(0.03, (state.brightness / 100) * 0.13)
              : 0;
            const style: KeyStyle = {
              left: percent(previewKey.x, BOARD_WIDTH),
              top: percent(previewKey.y, BOARD_HEIGHT),
              width: percent(previewKey.w, BOARD_WIDTH),
              height: percent(previewKey.h, BOARD_HEIGHT),
              "--key-color": rgbToCss(color),
              "--key-glow": rgbWithAlpha(color, glow),
              "--legend-color": rgbWithAlpha(color, active ? 0.72 : 0.32),
            };

            const onClick = editable
              ? () => onPaintSegment!(segmentForKey(previewKey, segmentCount!))
              : undefined;

            return (
              <div
                className={[
                  "keyboard-key",
                  `keyboard-key--${previewKey.align ?? "center"}`,
                  previewKey.compact ? "keyboard-key--compact" : "",
                  editable ? "keyboard-key--editable" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                key={previewKey.id}
                role={editable ? "button" : undefined}
                style={style}
                onClick={onClick}
              >
                <span className="keyboard-key__label">{previewKey.label}</span>
                {previewKey.subLabel && (
                  <span className="keyboard-key__sub-label">
                    {previewKey.subLabel}
                  </span>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </Card>
  );
}

function resolvePreviewPalette(state: KeyboardState): RgbColor[] {
  const zoneColors = state.zone_colors;
  if (
    !effectSupportsZoneColors(state.effect) ||
    !zoneColors ||
    zoneColors.length === 0
  ) {
    return [state.primary_color];
  }

  // The editor emits a dense palette indexed 0..n-1 (one entry per segment).
  // Anything else falls back to a single primary color across the board.
  const count = zoneColors.length;
  const palette: Array<RgbColor | null> = Array.from(
    { length: count },
    () => null,
  );

  for (const zone of zoneColors) {
    if (
      zone.zone_index < 0 ||
      zone.zone_index >= count ||
      palette[zone.zone_index] !== null
    ) {
      return [state.primary_color];
    }

    palette[zone.zone_index] = zone.color;
  }

  return palette.map((color) => color ?? state.primary_color);
}

function segmentForKey(previewKey: PreviewKey, segmentCount: number): number {
  const centerX = previewKey.x + previewKey.w / 2;
  return Math.min(
    segmentCount - 1,
    Math.max(0, Math.floor((centerX / BOARD_WIDTH) * segmentCount)),
  );
}

function percent(value: number, total: number): string {
  return `${(value / total) * 100}%`;
}

function rgbWithAlpha(color: RgbColor, alpha: number): string {
  return `rgb(${color.r} ${color.g} ${color.b} / ${alpha})`;
}
