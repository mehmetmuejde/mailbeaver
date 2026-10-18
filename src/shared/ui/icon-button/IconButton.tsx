import type { ComponentProps, ReactNode } from "react";
import { Button } from "@/shared/ui/primitives/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/primitives/tooltip";

type IconButtonProps = Omit<ComponentProps<"button">, "children" | "aria-label"> & {
  /** Accessible name and tooltip text. */
  label: string;
  icon: ReactNode;
  variant?: "ghost" | "destructive";
};

/** Square button with only an icon; the label is its accessible name and its tooltip. */
export function IconButton({
  label,
  icon,
  variant = "ghost",
  type = "button",
  ...props
}: IconButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button type={type} variant={variant} size="icon" aria-label={label} {...props}>
          <span aria-hidden="true">{icon}</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}
