import type { ChangeEvent } from "react";

export interface SelectOption<TValue extends string> {
  value: TValue;
  label: string;
  disabled?: boolean;
}

interface SelectProps<TValue extends string> {
  label: string;
  value: TValue;
  options: SelectOption<TValue>[];
  disabled?: boolean;
  onChange: (value: TValue) => void;
}

export function Select<TValue extends string>({
  label,
  value,
  options,
  disabled = false,
  onChange,
}: SelectProps<TValue>) {
  const handleChange = (event: ChangeEvent<HTMLSelectElement>) => {
    onChange(event.currentTarget.value as TValue);
  };

  return (
    <label className="field">
      <span>{label}</span>
      <select disabled={disabled} value={value} onChange={handleChange}>
        {options.map((option) => (
          <option
            disabled={option.disabled}
            key={option.value}
            value={option.value}
          >
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}
