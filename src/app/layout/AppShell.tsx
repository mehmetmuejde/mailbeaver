import { Outlet, useLocation } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { IS_DEV_BUILD } from "@/shared/config";
import { hasOverlayTitleBar } from "@/shared/lib/platform";
import { useSidebarCollapsed } from "@/shared/lib/sidebar";
import { Logo } from "@/shared/ui/logo";
import { SIDEBAR_ID } from "@/shared/ui/sidebar-toggle";
import { AppNavigation } from "./AppNavigation";
import { AppVersion } from "./AppVersion";
import { CapturePausedBanner } from "./CapturePausedBanner";
import { useMailboxEvents } from "./use-mailbox-events";
import { useOpenMessageRequests } from "./use-open-message-requests";
import { usePersistedPreferences } from "./use-persisted-preferences";

/** App frame: a sidebar that slides in and out (toggled from the page header). */
export function AppShell() {
  useMailboxEvents();
  useOpenMessageRequests();
  usePersistedPreferences();
  const collapsed = useSidebarCollapsed();
  const pathname = useLocation({ select: (location) => location.pathname });
  const main = useRef<HTMLElement>(null);
  const previousPathname = useRef(pathname);

  // After navigating, screen reader and keyboard users start at the new page.
  useEffect(() => {
    if (previousPathname.current !== pathname) {
      previousPathname.current = pathname;
      main.current?.focus();
    }
  }, [pathname]);

  return (
    <div className="flex h-full">
      <aside
        id={SIDEBAR_ID}
        aria-label="Sidebar"
        inert={collapsed}
        className={`flex shrink-0 flex-col overflow-hidden border-r bg-muted/40 transition-[width] duration-200 motion-reduce:transition-none ${
          collapsed ? "w-0 border-r-0" : "w-56"
        }`}
      >
        {/*
          Room for the macOS traffic lights, as high as the page header (h-14), so lights,
          sidebar toggle and title share one line. Drags the window like a title bar.
        */}
        <div data-tauri-drag-region className={hasOverlayTitleBar() ? "h-14 shrink-0" : "h-3"} />
        <div data-tauri-drag-region className="flex justify-center px-4 pt-2 pb-6">
          <p>
            <Logo dev={IS_DEV_BUILD} />
          </p>
        </div>
        <AppNavigation />
        <div className="mt-auto">
          <AppVersion />
        </div>
      </aside>
      <main ref={main} tabIndex={-1} className="flex min-h-0 min-w-0 flex-1 flex-col outline-none">
        <CapturePausedBanner />
        <div className="min-h-0 flex-1">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
