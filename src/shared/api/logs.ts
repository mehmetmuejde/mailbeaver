import { queryOptions } from "@tanstack/react-query";
import { z } from "zod";
import { apiUrl, request, requestJson } from "./client";

export const LOG_LEVELS = ["error", "warn", "info"] as const;
export const LOG_SOURCES = ["backend", "frontend"] as const;
export type LogLevel = (typeof LOG_LEVELS)[number];
export type LogSource = (typeof LOG_SOURCES)[number];

/** How often the open log tab looks for new entries. */
const LOG_REFRESH_MS = 3000;
/** The same entry is sent at most once in this time. */
const REPORT_DEDUPE_MS = 5000;
const REPORT_MAX_MESSAGE_LENGTH = 4000;

const logEntrySchema = z.object({
  id: z.number().int(),
  /** Unix milliseconds. */
  createdAt: z.number(),
  level: z.enum(LOG_LEVELS),
  source: z.enum(LOG_SOURCES),
  target: z.string(),
  message: z.string(),
});
export type LogEntry = z.infer<typeof logEntrySchema>;

export type LogFilter = {
  /** Only entries at least this severe. */
  minLevel: LogLevel;
  /** `null` shows both. */
  source: LogSource | null;
};

function fetchLogs(filter: LogFilter): Promise<LogEntry[]> {
  const params = new URLSearchParams({ minLevel: filter.minLevel });
  if (filter.source !== null) {
    params.set("source", filter.source);
  }
  return requestJson(`/api/logs?${params}`, z.array(logEntrySchema));
}

export async function clearLogs(): Promise<void> {
  await request("/api/logs", { method: "DELETE" });
}

export const logKeys = {
  all: ["logs"] as const,
  list: (filter: LogFilter) => [...logKeys.all, filter] as const,
};

export const logQueries = {
  list: (filter: LogFilter) =>
    queryOptions({
      queryKey: logKeys.list(filter),
      queryFn: () => fetchLogs(filter),
      refetchInterval: LOG_REFRESH_MS,
    }),
};

const lastSent = new Map<string, number>();

/**
 * Sends an entry of the UI to the server log (Settings → Logs). Never throws: a failed
 * report is dropped, so reporting can never cause further reports.
 */
export function reportLog(level: LogLevel, target: string, message: string): void {
  const key = `${level}|${target}|${message}`;
  const now = Date.now();
  const previous = lastSent.get(key);
  if (previous !== undefined && now - previous < REPORT_DEDUPE_MS) {
    return;
  }
  lastSent.set(key, now);
  fetch(apiUrl("/api/logs"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      level,
      target,
      message: message.slice(0, REPORT_MAX_MESSAGE_LENGTH),
    }),
  }).catch(() => undefined);
}
