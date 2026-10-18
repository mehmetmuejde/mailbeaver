import type { QueryClient } from "@tanstack/react-query";
import {
  createHashHistory,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  type RouterHistory,
  redirect,
} from "@tanstack/react-router";
import { createMailRoutes } from "@/pages/mail";
import { createSettingsRoute } from "@/pages/settings";
import { AppShell } from "./layout/AppShell";
import { NotFound } from "./layout/NotFound";
import { RouteError } from "./layout/RouteError";

type RouterContext = {
  queryClient: QueryClient;
};

type AppRouterOptions = RouterContext & {
  /** Hash history by default: works in the Tauri webview and from any static server. */
  history?: RouterHistory;
};

export function createAppRouter({ queryClient, history }: AppRouterOptions) {
  const rootRoute = createRootRouteWithContext<RouterContext>()({
    component: AppShell,
    errorComponent: RouteError,
    notFoundComponent: NotFound,
  });
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    beforeLoad: () => {
      throw redirect({ to: "/inbox" });
    },
  });
  const mail = createMailRoutes(rootRoute);
  const routeTree = rootRoute.addChildren([
    indexRoute,
    mail.inbox,
    mail.archive,
    mail.trash,
    createSettingsRoute(rootRoute),
  ]);

  return createRouter({
    routeTree,
    history: history ?? createHashHistory(),
    context: { queryClient },
    defaultPreload: "intent",
    // TanStack Query owns caching; the router always asks it.
    defaultPreloadStaleTime: 0,
  });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
