import type { RgbColor, ZoneColor } from "../../domain/lighting";
import { createDefaultZoneColors } from "../../domain/lighting";
import { ColorPicker } from "./ColorPicker";

interface ZoneColorEditorProps {
  zoneCount: number;
  primaryColor: RgbColor;
  zoneColors: ZoneColor[] | null;
  disabled?: boolean;
  onChange: (zoneColors: ZoneColor[] | null) => void;
}

/**
 * "Advanced zones" editor. When the "use same color for all zones" toggle is
 * on, per-zone colors are cleared (null) and the backend falls back to the
 * primary color for every zone. Zone indices are zero-based.
 */
export function ZoneColorEditor({
  zoneCount,
  primaryColor,
  zoneColors,
  disabled = false,
  onChange,
}: ZoneColorEditorProps) {
  const usePerZone = zoneColors !== null;

  const togglePerZone = (enabled: boolean) => {
    if (enabled) {
      onChange(zoneColors ?? createDefaultZoneColors(zoneCount, primaryColor));
    } else {
      onChange(null);
    }
  };

  const updateZone = (zoneIndex: number, color: RgbColor) => {
    const base = zoneColors ?? createDefaultZoneColors(zoneCount, primaryColor);
    onChange(
      base.map((zone) =>
        zone.zone_index === zoneIndex ? { ...zone, color } : zone,
      ),
    );
  };

  const effectiveZones =
    zoneColors ?? createDefaultZoneColors(zoneCount, primaryColor);

  return (
    <div className="zone-editor">
      <div className="zone-editor__header">
        <span>Advanced zones</span>
        <label className="zone-editor__toggle">
          <input
            checked={!usePerZone}
            disabled={disabled}
            type="checkbox"
            onChange={(event) => togglePerZone(!event.currentTarget.checked)}
          />
          <span>Use same color for all zones</span>
        </label>
      </div>
      {usePerZone && (
        <div className="zone-editor__grid">
          {effectiveZones.map((zone) => (
            <ColorPicker
              disabled={disabled}
              key={zone.zone_index}
              label={`Zone ${zone.zone_index + 1}`}
              value={zone.color}
              onChange={(color) => updateZone(zone.zone_index, color)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
