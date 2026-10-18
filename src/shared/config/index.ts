/** Default freshness of server data; live updates invalidate earlier. */
export const QUERY_STALE_TIME_MS = 30_000;

/** Retries for failed queries that are not client errors. */
export const QUERY_MAX_RETRIES = 2;

/** Pause after the last keystroke before a search runs. */
export const SEARCH_DEBOUNCE_MS = 300;

export const APP_NAME = "Mailbeaver";

export const FOLDER_LABELS = {
  inbox: "Inbox",
  archive: "Archive",
  trash: "Trash",
} as const;
