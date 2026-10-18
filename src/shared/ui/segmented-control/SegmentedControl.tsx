import type { ReactNode } from "react";
import { ToggleGroup, ToggleGroupItem } from "@/shared/ui/primitives/toggle-group";

export type SegmentedOption<T extends string> = {
  value: T;
  label: string;
  icon?: ReactNode;
};

type SegmentedControlProps<T extends string> = {
  /** Accessible name of the group. */
  label: string;
  options: readonly SegmentedOption<T>[];
  value: T;
  onValueChange: (value: T) => void;
  /** Stretches the group over the whole width; the options share it equally. */
  fullWidth?: boolean;
};

/** A joined button group where exactly one option is pressed. */
export function SegmentedControl<T extends string>({
  label,
  options,
  value,
  onValueChange,
  fullWidth = false,
}: SegmentedControlProps<T>) {
  const handleChange = (next: string) => {
    // Pressing the active option again would clear the group; one option always stays.
    const option = options.find((candidate) => candidate.value === next);
    if (option !== undefined) {
      onValueChange(option.value);
    }
  };

  return (
    <ToggleGroup
      type="single"
      variant="outline"
      size="sm"
      spacing={0}
      className={fullWidth ? "w-full" : undefined}
      aria-label={label}
      value={value}
      onValueChange={handleChange}
    >
      {options.map((option) => (
        <ToggleGroupItem
          key={option.value}
          value={option.value}
          className={fullWidth ? "flex-1" : undefined}
        >
          {option.icon === undefined ? null : <span aria-hidden="true">{option.icon}</span>}
          {option.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
