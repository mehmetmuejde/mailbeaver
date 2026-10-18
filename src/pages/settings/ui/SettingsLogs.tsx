import { useQuery } from "@tanstack/react-query";
import { Copy, Trash2 } from "lucide-react";
import { useState } from "react";
import {
  LOG_LEVELS,
  LOG_SOURCES,
  type LogEntry,
  type LogFilter,
  type LogLevel,
  type LogSource,
  logQueries,
} from "@/shared/api";
import { useCopyToClipboard } from "@/shared/lib/clipboard";
import { formatDateTime, toIsoString } from "@/shared/lib/format";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { useClearLogs } from "../api/use-clear-logs";

const ALL_SOURCES = "all";

const LEVEL_OPTIONS: readonly SelectOption[] = [
  { value: "info", label: "All levels" },
  { value: "warn", label: "Warnings and errors" },
  { value: "error", label: "Errors only" },
];

const SOURCE_LABELS: Record<LogSource, string> = { backend: "Backend", frontend: "UI" };

const SOURCE_OPTIONS: readonly SelectOption[] = [
  { value: ALL_SOURCES, label: "Backend and UI" },
  ...LOG_SOURCES.map((source) => ({ value: source, label: SOURCE_LABELS[source] })),
];

const LEVEL_STYLES: Record<LogLevel, string> = {
  error: "text-destructive",
  warn: "text-amber-600 dark:text-amber-400",
  info: "text-muted-foreground",
};

function isLevel(value: string): value is LogLevel {
  return LOG_LEVELS.some((level) => level === value);
}

function isSource(value: string): value is LogSource {
  return LOG_SOURCES.some((source) => source === value);
}

/** One line per entry, for pasting into an issue or a chat. */
function asText(entries: readonly LogEntry[]): string {
  return entries
    .map(
      (entry) =>
        `${toIsoString(entry.createdAt)} ${entry.level.toUpperCase()} [${entry.source}] ${entry.target}: ${entry.message}`,
    )
    .join("\n");
}

/** What the backend and the UI reported, newest first; refreshes while open. */
export function SettingsLogs() {
  const [filter, setFilter] = useState<LogFilter>({ minLevel: "info", source: null });
  const { data: entries, isError } = useQuery(logQueries.list(filter));
  const clear = useClearLogs();
  const { copy, result } = useCopyToClipboard();
  const isEmpty = entries === undefined || entries.length === 0;

  return (
    <section aria-labelledby="logs-heading" className="grid gap-4">
      <h2 id="logs-heading" className="text-base font-semibold">
        Logs
      </h2>
      <p className="text-muted-foreground">
        Received mail, notifications, failed requests and errors of the backend and the UI. The
        newest 10,000 entries are kept; mail content and passwords are never logged.
      </p>
      <div className="flex flex-wrap items-center gap-3">
        <SelectField
          label="Level"
          options={LEVEL_OPTIONS}
          value={filter.minLevel}
          onChange={(value) => {
            if (isLevel(value)) {
              setFilter({ ...filter, minLevel: value });
            }
          }}
        />
        <SelectField
          label="Source"
          options={SOURCE_OPTIONS}
          value={filter.source ?? ALL_SOURCES}
          onChange={(value) => setFilter({ ...filter, source: isSource(value) ? value : null })}
        />
        <div className="ml-auto flex gap-2">
          <Button
            size="sm"
            icon={<Copy />}
            disabled={isEmpty}
            onClick={() => copy(asText(entries ?? []))}
          >
            Copy
          </Button>
          <ConfirmDialog
            trigger={
              <Button size="sm" variant="destructive" icon={<Trash2 />} disabled={isEmpty}>
                Clear log
              </Button>
            }
            title="Clear the log?"
            description="Every log entry of the backend and the UI will be deleted."
            confirmLabel="Clear log"
            confirmIcon={<Trash2 />}
            onConfirm={() => clear.mutate()}
          />
        </div>
      </div>
      <p role="status" className="text-xs text-muted-foreground">
        {result?.kind === "copied" ? "Copied." : null}
      </p>
      {isError ? <p role="alert">The log could not be loaded.</p> : null}
      {clear.isError ? <p role="alert">The log could not be cleared.</p> : null}
      {entries !== undefined && entries.length === 0 ? (
        <p className="text-muted-foreground">No log entries.</p>
      ) : null}
      {entries !== undefined && entries.length > 0 ? (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-sm">
            <caption className="sr-only">Log entries, newest first</caption>
            <thead className="bg-muted/50 text-left text-xs text-muted-foreground">
              <tr>
                <th scope="col" className="px-3 py-2 font-medium">
                  Time
                </th>
                <th scope="col" className="px-3 py-2 font-medium">
                  Level
                </th>
                <th scope="col" className="px-3 py-2 font-medium">
                  Source
                </th>
                <th scope="col" className="px-3 py-2 font-medium">
                  Message
                </th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.id} className="border-t align-top">
                  <td className="px-3 py-2 whitespace-nowrap text-muted-foreground">
                    <time dateTime={toIsoString(entry.createdAt)}>
                      {formatDateTime(entry.createdAt)}
                    </time>
                  </td>
                  <td
                    className={`px-3 py-2 text-xs font-semibold uppercase ${LEVEL_STYLES[entry.level]}`}
                  >
                    {entry.level}
                  </td>
                  <td className="px-3 py-2 whitespace-nowrap">
                    {SOURCE_LABELS[entry.source]}
                    <span className="block font-mono text-xs text-muted-foreground">
                      {entry.target}
                    </span>
                  </td>
                  <td className="px-3 py-2 font-mono text-xs break-all whitespace-pre-wrap">
                    {entry.message}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}
    </section>
  );
}
