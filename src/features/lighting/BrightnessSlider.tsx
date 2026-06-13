import { Slider } from "../../shared/components/Slider";

interface BrightnessSliderProps {
  value: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function BrightnessSlider({
  value,
  disabled = false,
  onChange,
}: BrightnessSliderProps) {
  return (
    <Slider
      disabled={disabled}
      label="Brightness"
      value={value}
      onChange={onChange}
    />
  );
}
