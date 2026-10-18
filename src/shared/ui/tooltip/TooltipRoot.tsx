import type { ReactNode } from "react";
import { TooltipProvider } from "@/shared/ui/primitives/tooltip";

const TOOLTIP_DELAY_MS = 400;

/** Shares one tooltip delay across the app; icon buttons need it above them. */
export function TooltipRoot({ children }: { children: ReactNode }) {
  return <TooltipProvider delayDuration={TOOLTIP_DELAY_MS}>{children}</TooltipProvider>;
}
