import { useEffect, useMemo, useState } from "react";

import type { DeviceInfo } from "../../domain/device";
import {
  createDefaultKeyboardState,
  createDefaultZoneColors,
  EFFECT_DIRECTIONS,
  effectSupportsZoneColors,
  effectUsesDirection,
  effectUsesSpeed,
  isDirectionSupportedForEffect,
  type EffectDirection,
  type KeyboardState,
  type RgbColor,
  type ZoneColor,
} from "../../domain/lighting";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { hsvToRgb } from "../../shared/utils/color";
import { canApplyState } from "../../shared/utils/validation";
import { BrightnessSlider } from "./BrightnessSlider";
import { ColorPicker } from "./ColorPicker";
import { DirectionSelector } from "./DirectionSelector";
import { EffectSelector } from "./EffectSelector";
import { KeyboardPreview } from "./KeyboardPreview";
import { SegmentColorEditor } from "./SegmentColorEditor";
import { SpeedSlider } from "./SpeedSlider";

interface LightingPageProps {
  device: DeviceInfo | null;
  keyboardState: KeyboardState | null;
  loading: boolean;
  onApply: (state: KeyboardState) => void;
  onTurnOff: () => void;
}

export function LightingPage({
  device,
  keyboardState,
  loading,
  onApply,
  onTurnOff,
}: LightingPageProps) {
  const [draft, setDraft] = useState<KeyboardState>(
    keyboardState ?? createDefaultKeyboardState(),
  );
  const [brushColor, setBrushColor] = useState<RgbColor>(
    (keyboardState ?? createDefaultKeyboardState()).primary_color,
  );

  useEffect(() => {
    if (keyboardState) {
      setDraft(keyboardState);
    }
  }, [keyboardState]);

  const capabilities = device?.capabilities ?? null;
  const canApply = useMemo(
    () => canApplyState(draft, capabilities),
    [capabilities, draft],
  );
  const effectDisabled = loading || !device?.supported;
  const isOff = draft.effect === "Off";
  const disabledDirections: EffectDirection[] = EFFECT_DIRECTIONS.filter(
    (direction) => !isDirectionSupportedForEffect(draft.effect, direction),
  );
  const segmentCount = capabilities?.zone_count ?? 0;
  const showZones =
    !!capabilities?.supports_zones &&
    segmentCount > 1 &&
    effectSupportsZoneColors(draft.effect);
  const perSegment = draft.zone_colors !== null;
  const canPaint = showZones && !isOff && !loading;

  const setZoneColors = (zone_colors: ZoneColor[] | null) =>
    setDraft((current) => ({ ...current, zone_colors }));

  const paintSegment = (index: number) => {
    const base =
      draft.zone_colors ??
      createDefaultZoneColors(segmentCount, draft.primary_color);
    setZoneColors(
      base.map((segment) =>
        segment.zone_index === index
          ? { ...segment, color: { ...brushColor } }
          : segment,
      ),
    );
  };

  const fillAll = () =>
    setZoneColors(
      Array.from({ length: segmentCount }, (_unused, index) => ({
        zone_index: index,
        color: { ...brushColor },
      })),
    );

  const fillRainbow = () => {
    const last = Math.max(1, segmentCount - 1);
    setZoneColors(
      Array.from({ length: segmentCount }, (_unused, index) => ({
        zone_index: index,
        color: hsvToRgb(index / last, 1, 1),
      })),
    );
  };

  return (
    <section className="page-stack">
      <div className="page-heading">
        <div>
          <p className="eyebrow">Effects</p>
          <h1>Lighting controls</h1>
        </div>
        <div className="action-row">
          <Button
            disabled={loading || !canApply}
            variant="primary"
            onClick={() => onApply(draft)}
          >
            Apply
          </Button>
          <Button disabled={loading} variant="danger" onClick={onTurnOff}>
            Turn off
          </Button>
        </div>
      </div>

      {!device?.supported && (
        <Notice tone="warning">
          The active device does not report supported lighting capabilities.
        </Notice>
      )}

      {capabilities && !capabilities.supports_reactive && (
        <Notice tone="info">
          Reactive is disabled because the active backend does not support it yet.
        </Notice>
      )}

      {effectUsesDirection(draft.effect) && disabledDirections.length > 0 && (
        <Notice tone="info">
          Vertical wave directions are disabled until the Lenovo HID direction bytes are verified.
        </Notice>
      )}

      <KeyboardPreview
        state={draft}
        segmentCount={segmentCount}
        onPaintSegment={canPaint ? paintSegment : undefined}
      />

      <Card>
        <div className="control-grid">
          <EffectSelector
            capabilities={capabilities}
            disabled={effectDisabled}
            value={draft.effect}
            onChange={(effect) =>
              setDraft((current) => ({
                ...current,
                effect,
                enabled: effect !== "Off",
                brightness: effect === "Off" ? 0 : Math.max(current.brightness, 1),
              }))
            }
          />
          <BrightnessSlider
            disabled={loading || isOff || !capabilities?.supports_brightness}
            value={draft.brightness}
            onChange={(brightness) =>
              setDraft((current) => ({ ...current, brightness }))
            }
          />
          <SpeedSlider
            disabled={
              loading ||
              isOff ||
              !effectUsesSpeed(draft.effect) ||
              !capabilities?.supports_speed
            }
            value={draft.speed}
            onChange={(speed) => setDraft((current) => ({ ...current, speed }))}
          />
          <DirectionSelector
            disabledDirections={disabledDirections}
            disabled={
              loading ||
              isOff ||
              !effectUsesDirection(draft.effect) ||
              !capabilities?.supports_direction
            }
            value={draft.direction}
            onChange={(direction) =>
              setDraft((current) => ({ ...current, direction }))
            }
          />
          <ColorPicker
            disabled={loading || isOff || !capabilities?.supports_primary_color}
            label="Primary color"
            value={draft.primary_color}
            onChange={(primary_color) =>
              setDraft((current) => ({ ...current, primary_color }))
            }
          />
          <ColorPicker
            disabled={loading || isOff || !capabilities?.supports_secondary_color}
            label="Secondary color"
            value={draft.secondary_color ?? draft.primary_color}
            onChange={(secondary_color) =>
              setDraft((current) => ({ ...current, secondary_color }))
            }
          />
        </div>
        {showZones && (
          <SegmentColorEditor
            disabled={loading || isOff}
            segmentCount={segmentCount}
            perSegment={perSegment}
            brushColor={brushColor}
            onBrushChange={setBrushColor}
            onTogglePerSegment={(enabled) =>
              setZoneColors(
                enabled
                  ? createDefaultZoneColors(segmentCount, draft.primary_color)
                  : null,
              )
            }
            onFillAll={fillAll}
            onRainbow={fillRainbow}
          />
        )}
      </Card>
    </section>
  );
}
