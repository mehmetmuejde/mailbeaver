import { z } from "zod";

export const SETTINGS_TABS = [
  "general",
  "projects",
  "trash",
  "tray",
  "notifications",
  "stats",
  "api",
  "logs",
  "danger",
] as const;
export type SettingsTab = (typeof SETTINGS_TABS)[number];

/** URL state of the settings page; an unknown tab falls back to the first one. */
export const settingsSearchSchema = z.object({
  tab: z.enum(SETTINGS_TABS).optional().catch(undefined),
});
