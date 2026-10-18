import { type ChangeEvent, useId } from "react";
import { Label } from "@/shared/ui/primitives/label";
import { NativeSelect, NativeSelectOption } from "@/shared/ui/primitives/native-select";

export type SelectOption = {
  value: string;
  label: string;
};

type SelectFieldProps = {
  label: string;
  /** Keeps the label for screen readers only, e.g. inside a toolbar. */
  hideLabel?: boolean;
  options: readonly SelectOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  /** Label at the start, a fixed-width select at the end: aligned rows in forms. */
  spread?: boolean;
};

export function SelectField({
  label,
  hideLabel = false,
  options,
  value,
  onChange,
  disabled,
  spread = false,
}: SelectFieldProps) {
  const id = useId();
  return (
    <div className={`flex items-center gap-2 ${spread ? "justify-between" : ""}`}>
      <Label htmlFor={id} className={hideLabel ? "sr-only" : undefined}>
        {label}
      </Label>
      <NativeSelect
        id={id}
        className={spread ? "w-36" : undefined}
        value={value}
        disabled={disabled}
        onChange={(event: ChangeEvent<HTMLSelectElement>) => onChange(event.target.value)}
      >
        {options.map((option) => (
          <NativeSelectOption key={option.value} value={option.value}>
            {option.label}
          </NativeSelectOption>
        ))}
      </NativeSelect>
    </div>
  );
}
