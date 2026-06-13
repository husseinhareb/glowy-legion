import type { RgbColor } from "../../domain/lighting";
import { hexToRgb, rgbToHex } from "../../shared/utils/color";

interface ColorPickerProps {
  label: string;
  value: RgbColor;
  disabled?: boolean;
  onChange: (value: RgbColor) => void;
}

export function ColorPicker({
  label,
  value,
  disabled = false,
  onChange,
}: ColorPickerProps) {
  return (
    <label className="field field--color">
      <span>{label}</span>
      <span className="color-control">
        <input
          disabled={disabled}
          type="color"
          value={rgbToHex(value)}
          onChange={(event) => onChange(hexToRgb(event.currentTarget.value))}
        />
        <code>{rgbToHex(value).toUpperCase()}</code>
      </span>
    </label>
  );
}
