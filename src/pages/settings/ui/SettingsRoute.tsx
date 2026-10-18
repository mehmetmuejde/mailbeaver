import { useNavigate, useSearch } from "@tanstack/react-router";
import { settingsSearchSchema } from "../model/settings-search";
import { SettingsPage } from "./SettingsPage";

/** Loaded on demand: the settings are a separate chunk of the bundle. */
export function SettingsRoute() {
  // Parsed again for the types the generic factory erases (see create-mail-routes.tsx).
  const search = settingsSearchSchema.parse(useSearch({ strict: false }));
  const navigate = useNavigate();
  return (
    <SettingsPage
      tab={search.tab ?? "general"}
      onTabChange={(tab) =>
        void navigate({
          to: ".",
          search: (prev) => ({ ...prev, tab: tab === "general" ? undefined : tab }),
          replace: true,
        })
      }
    />
  );
}
