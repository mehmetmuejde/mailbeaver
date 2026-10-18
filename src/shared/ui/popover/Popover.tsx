import type { ReactNode } from "react";
import {
  PopoverContent,
  Popover as PopoverRoot,
  PopoverTrigger,
} from "@/shared/ui/primitives/popover";

type PopoverProps = {
  /** The button that opens the popover; focus returns to it on close. */
  trigger: ReactNode;
  /** Accessible name of the popover panel. */
  label: string;
  /** `wide` fits a short reference table. */
  width?: "default" | "wide";
  children: ReactNode;
};

/** A panel below its trigger; Escape or a click outside closes it. */
export function Popover({ trigger, label, width = "default", children }: PopoverProps) {
  return (
    <PopoverRoot>
      <PopoverTrigger asChild>{trigger}</PopoverTrigger>
      <PopoverContent
        align="end"
        aria-label={label}
        className={`grid gap-3 p-3 ${width === "wide" ? "w-80" : "w-64"}`}
      >
        {children}
      </PopoverContent>
    </PopoverRoot>
  );
}
