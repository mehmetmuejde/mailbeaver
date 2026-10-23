/** Default freshness of server data; live updates invalidate earlier. */
export const QUERY_STALE_TIME_MS = 30_000;

/** Retries for failed queries that are not client errors. */
export const QUERY_MAX_RETRIES = 2;

/** Pause after the last keystroke before a search runs. */
export const SEARCH_DEBOUNCE_MS = 300;

/**
 * True under `pnpm tauri dev` / `pnpm dev`, so a development build is never mistaken for
 * the installed app. Tests run in mode `test` and see the release names.
 */
export const IS_DEV_BUILD = import.meta.env.MODE === "development";

export const APP_NAME = IS_DEV_BUILD ? "Mailbeaver (Dev)" : "Mailbeaver";

export const FOLDER_LABELS = {
  inbox: "Inbox",
  archive: "Archive",
  trash: "Trash",
} as const;
