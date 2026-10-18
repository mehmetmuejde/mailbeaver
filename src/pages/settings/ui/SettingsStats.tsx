import { useQuery } from "@tanstack/react-query";
import { RotateCw } from "lucide-react";
import { settingsQueries } from "@/shared/api";
import { formatBytes, formatDateTime, formatRelativeTime, pluralize } from "@/shared/lib/format";
import { Button } from "@/shared/ui/button";

/** Figures since the app started, plus the size of the database file. */
export function SettingsStats() {
  const { data: stats, isError, isFetching, refetch } = useQuery(settingsQueries.stats());

  return (
    <section aria-labelledby="stats-heading" className="grid gap-3">
      <div className="flex items-center justify-between gap-4">
        <h2 id="stats-heading" className="text-base font-semibold">
          Statistics
        </h2>
        <Button size="sm" icon={<RotateCw />} disabled={isFetching} onClick={() => void refetch()}>
          Refresh
        </Button>
      </div>
      {isError ? <p role="alert">Statistics could not be loaded.</p> : null}
      {stats === undefined ? null : (
        <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
          <dt className="text-muted-foreground">Running since</dt>
          <dd>
            <time
              dateTime={new Date(stats.startedAt).toISOString()}
              title={formatDateTime(stats.startedAt)}
            >
              {formatRelativeTime(stats.startedAt)}
            </time>
          </dd>
          <dt className="text-muted-foreground">Received via SMTP</dt>
          <dd>
            {pluralize(stats.smtpAccepted, "message")} ({formatBytes(stats.smtpAcceptedBytes)})
          </dd>
          <dt className="text-muted-foreground">Rejected via SMTP</dt>
          <dd>{pluralize(stats.smtpRejected, "message")}</dd>
          <dt className="text-muted-foreground">Deleted</dt>
          <dd>{pluralize(stats.messagesDeleted, "message")}</dd>
          <dt className="text-muted-foreground">Database size</dt>
          <dd>{formatBytes(stats.databaseBytes)}</dd>
        </dl>
      )}
    </section>
  );
}
