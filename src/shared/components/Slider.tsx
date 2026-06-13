interface SliderProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function Slider({
  label,
  value,
  min = 0,
  max = 100,
  disabled = false,
  onChange,
}: SliderProps) {
  return (
    <label className="field">
      <span className="field__label-row">
        <span>{label}</span>
        <strong>{value}</strong>
      </span>
      <input
        disabled={disabled}
        max={max}
        min={min}
        type="range"
        value={value}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </label>
  );
}
