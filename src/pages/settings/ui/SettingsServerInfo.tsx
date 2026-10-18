import { useQuery } from "@tanstack/react-query";
import { SmtpSetup } from "@/entities/smtp-server";
import { settingsQueries } from "@/shared/api";
import { formatBytes } from "@/shared/lib/format";

/** How to reach Mailbeaver, and where and with which limits it runs. */
export function SettingsServerInfo() {
  const { data: info, isPending, isError } = useQuery(settingsQueries.info());

  return (
    <>
      <section aria-labelledby="connection-heading" className="grid gap-3">
        <h2 id="connection-heading" className="text-base font-semibold">
          Send mail to Mailbeaver
        </h2>
        <p className="text-muted-foreground">
          Use these SMTP settings in the application you are developing.
        </p>
        {isPending ? <p role="status">Loading server information…</p> : null}
        {isError ? <p role="alert">Server information could not be loaded.</p> : null}
        {info === undefined ? null : <SmtpSetup address={info.smtpAddress} />}
      </section>
      {info === undefined ? null : (
        <section aria-labelledby="server-heading" className="grid gap-3">
          <h2 id="server-heading" className="text-base font-semibold">
            Server
          </h2>
          <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
            <dt className="text-muted-foreground">Version</dt>
            <dd>{info.version}</dd>
            <dt className="text-muted-foreground">HTTP API</dt>
            <dd>
              <code className="font-mono">{info.httpAddress}</code>
            </dd>
            <dt className="text-muted-foreground">Database</dt>
            <dd>
              <code className="font-mono break-all">{info.database}</code>
            </dd>
            <dt className="text-muted-foreground">Kept messages</dt>
            <dd>The newest {info.maxMessages}</dd>
            <dt className="text-muted-foreground">Maximum size</dt>
            <dd>{formatBytes(info.maxMessageBytes)} per message</dd>
          </dl>
        </section>
      )}
    </>
  );
}
