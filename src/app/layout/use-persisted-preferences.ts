import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { settingsKeys, settingsQueries, updateSettings } from "@/shared/api";
import { setSidebarCollapsed, useSidebarCollapsed } from "@/shared/lib/sidebar";
import { setThemePreference, useThemePreference } from "@/shared/lib/theme";

/**
 * Keeps theme and sidebar in the database (ADR-0024). The local copies make the first
 * paint right; once the settings arrive they win, and every later change is saved.
 */
export function usePersistedPreferences(): void {
  const queryClient = useQueryClient();
  const { data: settings } = useQuery(settingsQueries.settings());
  const theme = useThemePreference();
  const sidebarCollapsed = useSidebarCollapsed();
  const save = useMutation({
    mutationFn: updateSettings,
    onSuccess: (saved) => queryClient.setQueryData(settingsKeys.settings(), saved),
  });
  const loaded = useRef(false);
  const { mutate } = save;

  useEffect(() => {
    if (settings === undefined) {
      return;
    }
    if (!loaded.current) {
      loaded.current = true;
      setThemePreference(settings.theme);
      setSidebarCollapsed(settings.sidebarCollapsed);
      return;
    }
    if (settings.theme !== theme || settings.sidebarCollapsed !== sidebarCollapsed) {
      mutate({ ...settings, theme, sidebarCollapsed });
    }
  }, [settings, theme, sidebarCollapsed, mutate]);
}
