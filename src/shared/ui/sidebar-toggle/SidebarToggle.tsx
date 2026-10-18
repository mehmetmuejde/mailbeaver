import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { setSidebarCollapsed, useSidebarCollapsed } from "@/shared/lib/sidebar";
import { IconButton } from "@/shared/ui/icon-button";

/** Id of the app's sidebar, which the toggle controls. */
export const SIDEBAR_ID = "app-sidebar";

/** Slides the sidebar in or out; it sits at the start of every page header. */
export function SidebarToggle() {
  const collapsed = useSidebarCollapsed();
  return (
    <span className="inline-flex">
      <IconButton
        label={collapsed ? "Show sidebar" : "Hide sidebar"}
        icon={collapsed ? <PanelLeftOpen /> : <PanelLeftClose />}
        aria-expanded={!collapsed}
        aria-controls={SIDEBAR_ID}
        onClick={() => setSidebarCollapsed(!collapsed)}
      />
    </span>
  );
}
