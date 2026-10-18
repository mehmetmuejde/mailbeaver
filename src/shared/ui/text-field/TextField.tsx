import { type ChangeEvent, useId } from "react";
import { Input } from "@/shared/ui/primitives/input";
import { Label } from "@/shared/ui/primitives/label";

type TextFieldProps = {
  label: string;
  /** Keeps the label for screen readers only, e.g. inside a row of fields. */
  hideLabel?: boolean;
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  placeholder?: string;
  maxLength?: number;
  /** Shown below the field and announced with it. */
  error?: string;
  /** A hint below the field, announced with it. */
  description?: string;
  disabled?: boolean;
  inputMode?: "text" | "numeric";
  /** Values the browser offers while typing; the label adds context such as a count. */
  suggestions?: readonly { value: string; label: string }[];
};

export function TextField({
  label,
  hideLabel = false,
  value,
  onChange,
  onBlur,
  placeholder,
  maxLength,
  error,
  description,
  disabled,
  inputMode,
  suggestions,
}: TextFieldProps) {
  const id = useId();
  const errorId = `${id}-error`;
  const descriptionId = `${id}-description`;
  const describedBy = [
    description === undefined ? null : descriptionId,
    error === undefined ? null : errorId,
  ]
    .filter((part) => part !== null)
    .join(" ");
  const listId = `${id}-suggestions`;
  return (
    <div className="grid flex-1 gap-1">
      <Label htmlFor={id} className={hideLabel ? "sr-only" : undefined}>
        {label}
      </Label>
      <Input
        id={id}
        value={value}
        placeholder={placeholder}
        maxLength={maxLength}
        disabled={disabled}
        inputMode={inputMode}
        list={suggestions === undefined ? undefined : listId}
        autoComplete={suggestions === undefined ? undefined : "off"}
        aria-invalid={error === undefined ? undefined : true}
        aria-describedby={describedBy === "" ? undefined : describedBy}
        onBlur={onBlur}
        onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
      />
      {suggestions === undefined ? null : (
        <datalist id={listId}>
          {suggestions.map((suggestion) => (
            <option key={suggestion.value} value={suggestion.value} label={suggestion.label} />
          ))}
        </datalist>
      )}
      {description === undefined ? null : (
        <p id={descriptionId} className="text-xs text-muted-foreground">
          {description}
        </p>
      )}
      {error === undefined ? null : (
        <p id={errorId} className="text-xs text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
