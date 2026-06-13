import type { DeviceCapabilities } from "../../domain/device";
import {
  EFFECT_LABELS,
  LIGHTING_EFFECTS,
  isEffectSupported,
  type LightingEffect,
} from "../../domain/lighting";
import { Select } from "../../shared/components/Select";

interface EffectSelectorProps {
  value: LightingEffect;
  capabilities: DeviceCapabilities | null;
  disabled?: boolean;
  onChange: (value: LightingEffect) => void;
}

export function EffectSelector({
  value,
  capabilities,
  disabled = false,
  onChange,
}: EffectSelectorProps) {
  return (
    <Select
      disabled={disabled}
      label="Effect"
      options={LIGHTING_EFFECTS.map((effect) => ({
        value: effect,
        label: EFFECT_LABELS[effect],
        disabled: !isEffectSupported(effect, capabilities),
      }))}
      value={value}
      onChange={onChange}
    />
  );
}
