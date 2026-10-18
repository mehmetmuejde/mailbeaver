import { Search } from "lucide-react";
import { type ChangeEvent, type KeyboardEvent, useId } from "react";
import { Input } from "@/shared/ui/primitives/input";

type SearchFieldProps = {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  maxLength?: number;
  /** Called when Escape is pressed in the field. */
  onEscape?: () => void;
};

export function SearchField({
  label,
  value,
  onChange,
  placeholder,
  maxLength,
  onEscape,
}: SearchFieldProps) {
  const id = useId();
  return (
    <div className="relative w-full max-w-sm">
      <label htmlFor={id} className="sr-only">
        {label}
      </label>
      <Search
        aria-hidden="true"
        className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        id={id}
        type="search"
        className="pl-8"
        value={value}
        placeholder={placeholder}
        maxLength={maxLength}
        onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
        onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
          if (event.key === "Escape" && onEscape !== undefined) {
            event.preventDefault();
            onEscape();
          }
        }}
      />
    </div>
  );
}
