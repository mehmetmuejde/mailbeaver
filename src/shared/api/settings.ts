import { queryOptions } from "@tanstack/react-query";
import { z } from "zod";
import { requestJson, sendJson } from "./client";

const TRASH_RETENTION_MAX_DAYS = 365;

export const TRAY_GROUPINGS = ["all", "sender", "project"] as const;
export const THEMES = ["system", "light", "dark"] as const;
export const NOTIFICATION_MODES = ["off", "all", "projects"] as const;
export const PERIODS = ["today", "7d", "30d"] as const;
export const PAGE_SIZES = [25, 50, 100] as const;

const settingsSchema = z.object({
  /** `null` keeps messages in Trash until it is emptied by hand. */
  trashRetentionDays: z.number().int().min(1).max(TRASH_RETENTION_MAX_DAYS).nullable(),
  /** How the tray menu groups the newest inbox messages (desktop app). */
  trayGrouping: z.enum(TRAY_GROUPINGS),
  theme: z.enum(THEMES),
  sidebarCollapsed: z.boolean(),
  /** Which new mail raises a system notification (desktop app). */
  notifications: z.enum(NOTIFICATION_MODES),
  notificationSound: z.boolean(),
  /** Unread inbox count on the app icon (desktop app). */
  unreadBadge: z.boolean(),
  /** Asks GitHub once a day for a newer release; off by default (ADR-0032). */
  updateCheck: z.boolean(),
});
export type Settings = z.infer<typeof settingsSchema>;

const serverInfoSchema = z.object({
  version: z.string(),
  smtpAddress: z.string(),
  httpAddress: z.string(),
  database: z.string(),
  maxMessages: z.number(),
  maxMessageBytes: z.number(),
  /** Server settings fixed by an environment variable. */
  fromEnvironment: z.array(z.enum(["smtpPort", "httpPort", "maxMessages", "maxMessageMb"])),
  /** The desktop app can restart itself; the headless server cannot. */
  host: z.enum(["desktop", "server"]),
});
type ServerInfo = z.infer<typeof serverInfoSchema>;

/** Ports and limits for the next start; `null` keeps the default (ADR-0031). */
const serverSettingsSchema = z.object({
  smtpPort: z.number().int().nullable(),
  httpPort: z.number().int().nullable(),
  maxMessages: z.number().int().nullable(),
  maxMessageMb: z.number().int().nullable(),
});
export type ServerSettings = z.infer<typeof serverSettingsSchema>;

function fetchServerSettings(): Promise<ServerSettings> {
  return requestJson("/api/server-settings", serverSettingsSchema);
}

export async function saveServerSettings(settings: ServerSettings): Promise<ServerSettings> {
  const response = await sendJson("/api/server-settings", "PUT", settings);
  return serverSettingsSchema.parse(await response.json());
}

const folderViewSchema = z.object({
  grouping: z.enum(TRAY_GROUPINGS),
  /** `"none"` or a project id as text; `null` for every project. */
  project: z.string().nullable(),
  period: z.enum(PERIODS).nullable(),
  pageSize: z.union(PAGE_SIZES.map((size) => z.literal(size))),
});
export type FolderView = z.infer<typeof folderViewSchema>;

const folderViewsSchema = z.object({
  inbox: folderViewSchema,
  archive: folderViewSchema,
  trash: folderViewSchema,
});
export type FolderViews = z.infer<typeof folderViewsSchema>;

function fetchFolderViews(): Promise<FolderViews> {
  return requestJson("/api/folder-views", folderViewsSchema);
}

export async function saveFolderView({
  folder,
  view,
}: {
  folder: keyof FolderViews;
  view: FolderView;
}): Promise<FolderView> {
  const response = await sendJson(`/api/folder-views/${folder}`, "PUT", view);
  return folderViewSchema.parse(await response.json());
}

const statsSchema = z.object({
  /** Unix milliseconds. */
  startedAt: z.number(),
  smtpAccepted: z.number(),
  smtpAcceptedBytes: z.number(),
  smtpRejected: z.number(),
  messagesDeleted: z.number(),
  databaseBytes: z.number(),
});
type Stats = z.infer<typeof statsSchema>;

const updateStatusSchema = z.object({
  currentVersion: z.string(),
  latestVersion: z.string().nullable(),
  url: z.string().nullable(),
  updateAvailable: z.boolean(),
  checkedAt: z.number().nullable(),
});
type UpdateStatus = z.infer<typeof updateStatusSchema>;

function fetchUpdateStatus(): Promise<UpdateStatus> {
  return requestJson("/api/update", updateStatusSchema);
}

function fetchStats(): Promise<Stats> {
  return requestJson("/api/stats", statsSchema);
}

function fetchSettings(): Promise<Settings> {
  return requestJson("/api/settings", settingsSchema);
}

function fetchServerInfo(): Promise<ServerInfo> {
  return requestJson("/api/info", serverInfoSchema);
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  const response = await sendJson("/api/settings", "PUT", settings);
  return settingsSchema.parse(await response.json());
}

export const settingsKeys = {
  all: ["settings"] as const,
  settings: () => [...settingsKeys.all, "values"] as const,
  info: () => [...settingsKeys.all, "info"] as const,
  stats: () => [...settingsKeys.all, "stats"] as const,
  folderViews: () => [...settingsKeys.all, "folder-views"] as const,
  server: () => [...settingsKeys.all, "server"] as const,
  update: () => [...settingsKeys.all, "update"] as const,
};

export const settingsQueries = {
  folderViews: () =>
    queryOptions({ queryKey: settingsKeys.folderViews(), queryFn: fetchFolderViews }),
  stats: () => queryOptions({ queryKey: settingsKeys.stats(), queryFn: fetchStats }),
  server: () => queryOptions({ queryKey: settingsKeys.server(), queryFn: fetchServerSettings }),
  // The server checks in the background; the answer is picked up within a minute.
  update: () =>
    queryOptions({
      queryKey: settingsKeys.update(),
      queryFn: fetchUpdateStatus,
      refetchInterval: 60_000,
    }),
  settings: () => queryOptions({ queryKey: settingsKeys.settings(), queryFn: fetchSettings }),
  // Addresses and limits are fixed while the server runs.
  info: () =>
    queryOptions({
      queryKey: settingsKeys.info(),
      queryFn: fetchServerInfo,
      staleTime: Number.POSITIVE_INFINITY,
    }),
};
