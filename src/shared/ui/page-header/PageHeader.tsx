import type { ReactNode } from "react";
import { hasOverlayTitleBar } from "@/shared/lib/platform";
import { useSidebarCollapsed } from "@/shared/lib/sidebar";
import { SidebarToggle } from "@/shared/ui/sidebar-toggle";

type PageHeaderProps = {
  title: string;
  /** Counts, filters and actions, laid out after the title. */
  children?: ReactNode;
};

/**
 * The header every page starts with, led by the sidebar toggle. In the desktop app it
 * doubles as the title bar: empty space drags the window, and while the sidebar is
 * hidden it leaves room for the macOS traffic lights.
 */
export function PageHeader({ title, children }: PageHeaderProps) {
  const collapsed = useSidebarCollapsed();
  const insetForTrafficLights = collapsed && hasOverlayTitleBar();
  return (
    <header
      data-tauri-drag-region
      // 56 px high (h-14): tauri.conf.json centers the traffic lights on the same line.
      className={`flex min-h-14 flex-wrap items-center gap-3 border-b py-2 pr-4 ${
        insetForTrafficLights ? "pl-22" : "pl-4"
      }`}
    >
      <SidebarToggle />
      <h1 data-tauri-drag-region className="text-lg font-semibold">
        {title}
      </h1>
      {children}
    </header>
  );
}
