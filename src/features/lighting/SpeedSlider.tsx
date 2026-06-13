import { Slider } from "../../shared/components/Slider";

interface SpeedSliderProps {
  value: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function SpeedSlider({
  value,
  disabled = false,
  onChange,
}: SpeedSliderProps) {
  return (
    <Slider disabled={disabled} label="Animation speed" value={value} onChange={onChange} />
  );
}
