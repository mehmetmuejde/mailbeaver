import { type AnyRoute, createRoute, lazyRouteComponent } from "@tanstack/react-router";
import { settingsSearchSchema } from "../model/settings-search";

export function createSettingsRoute<TParent extends AnyRoute>(parentRoute: TParent) {
  return createRoute({
    getParentRoute: () => parentRoute,
    path: "/settings",
    validateSearch: settingsSearchSchema,
    // Settings are opened rarely; their code loads with the first visit (or on hover).
    component: lazyRouteComponent(() => import("./SettingsRoute"), "SettingsRoute"),
  });
}
