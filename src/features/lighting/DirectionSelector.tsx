import {
  DIRECTION_LABELS,
  EFFECT_DIRECTIONS,
  type EffectDirection,
} from "../../domain/lighting";
import { Select } from "../../shared/components/Select";

interface DirectionSelectorProps {
  value: EffectDirection;
  disabled?: boolean;
  disabledDirections?: EffectDirection[];
  onChange: (value: EffectDirection) => void;
}

export function DirectionSelector({
  value,
  disabled = false,
  disabledDirections = [],
  onChange,
}: DirectionSelectorProps) {
  return (
    <Select
      disabled={disabled}
      label="Direction"
      options={EFFECT_DIRECTIONS.map((direction) => ({
        value: direction,
        label: DIRECTION_LABELS[direction],
        disabled: disabledDirections.includes(direction),
      }))}
      value={value}
      onChange={onChange}
    />
  );
}
