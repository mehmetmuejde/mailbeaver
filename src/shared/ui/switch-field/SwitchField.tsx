import { useId } from "react";
import { Label } from "@/shared/ui/primitives/label";
import { Switch } from "@/shared/ui/primitives/switch";

type SwitchFieldProps = {
  label: string;
  /** One line under the label explaining the effect. */
  description?: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
};

/** An on/off setting: label and description at the start, the switch at the end. */
export function SwitchField({
  label,
  description,
  checked,
  onCheckedChange,
  disabled,
}: SwitchFieldProps) {
  const id = useId();
  const descriptionId = useId();
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="grid gap-1">
        <Label htmlFor={id}>{label}</Label>
        {description === undefined ? null : (
          <p id={descriptionId} className="text-sm text-muted-foreground">
            {description}
          </p>
        )}
      </div>
      <Switch
        id={id}
        checked={checked}
        disabled={disabled}
        aria-describedby={description === undefined ? undefined : descriptionId}
        onCheckedChange={onCheckedChange}
      />
    </div>
  );
}
