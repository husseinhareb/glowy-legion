import type { RgbColor } from "../../domain/lighting";
import { Button } from "../../shared/components/Button";
import { ColorPicker } from "./ColorPicker";

interface SegmentColorEditorProps {
  /** Number of independently addressable color segments (lamps). */
  segmentCount: number;
  /** Whether per-segment colors are currently in use. */
  perSegment: boolean;
  /** The brush color applied when clicking a key on the preview. */
  brushColor: RgbColor;
  disabled?: boolean;
  onBrushChange: (color: RgbColor) => void;
  onTogglePerSegment: (enabled: boolean) => void;
  onFillAll: () => void;
  onRainbow: () => void;
}

/**
 * Brush toolbar for per-lamp painting. The keyboard preview itself is the
 * canvas: pick a brush color here, then click keys to paint their segment.
 * "Fill all" paints every segment with the brush; "Rainbow" spreads a hue
 * gradient across all segments.
 */
export function SegmentColorEditor({
  segmentCount,
  perSegment,
  brushColor,
  disabled = false,
  onBrushChange,
  onTogglePerSegment,
  onFillAll,
  onRainbow,
}: SegmentColorEditorProps) {
  return (
    <div className="segment-editor">
      <div className="segment-editor__header">
        <span>Per-lamp painting ({segmentCount} segments)</span>
        <label className="segment-editor__toggle">
          <input
            checked={!perSegment}
            disabled={disabled}
            type="checkbox"
            onChange={(event) => onTogglePerSegment(!event.currentTarget.checked)}
          />
          <span>Use one color everywhere</span>
        </label>
      </div>

      <p className="segment-editor__hint">
        Click any key on the keyboard above to paint its segment with the brush
        color.
      </p>

      <div className="segment-editor__tools">
        <ColorPicker
          disabled={disabled}
          label="Brush color"
          value={brushColor}
          onChange={onBrushChange}
        />
        <div className="segment-editor__actions">
          <Button disabled={disabled} variant="ghost" onClick={onFillAll}>
            Fill all
          </Button>
          <Button disabled={disabled} variant="ghost" onClick={onRainbow}>
            Rainbow
          </Button>
        </div>
      </div>
    </div>
  );
}
