import { Copy } from "lucide-react";
import { useCopyToClipboard } from "@/shared/lib/clipboard";
import { Button } from "@/shared/ui/button";
import { IconButton } from "@/shared/ui/icon-button";
import { splitAddress } from "../lib/split-address";

type SmtpSetupProps = {
  /** Where the SMTP server listens, e.g. `127.0.0.1:1026`. */
  address: string;
};

/** The SMTP settings an application needs to send its mail to Mailbeaver. */
export function SmtpSetup({ address }: SmtpSetupProps) {
  const { host, port } = splitAddress(address);
  const { copy, result } = useCopyToClipboard();
  const envSnippet = `SMTP_HOST=${host}\nSMTP_PORT=${port}`;

  return (
    <div className="grid gap-3 rounded-lg border bg-card p-4 text-card-foreground">
      <dl className="grid grid-cols-[max-content_1fr_auto] items-center gap-x-4 gap-y-1">
        <dt className="text-muted-foreground">Host</dt>
        <dd>
          <code className="font-mono">{host}</code>
        </dd>
        <dd>
          <IconButton label="Copy host" icon={<Copy />} onClick={() => copy(host)} />
        </dd>
        <dt className="text-muted-foreground">Port</dt>
        <dd>
          <code className="font-mono">{port}</code>
        </dd>
        <dd>
          <IconButton label="Copy port" icon={<Copy />} onClick={() => copy(port)} />
        </dd>
        <dt className="text-muted-foreground">Encryption</dt>
        <dd className="col-span-2">None (no TLS)</dd>
        <dt className="text-muted-foreground">Username</dt>
        <dd className="col-span-2">Optional – sorts the mail into a project</dd>
        <dt className="text-muted-foreground">Password</dt>
        <dd className="col-span-2">Any value – never stored</dd>
      </dl>
      <div className="flex flex-wrap items-center gap-3">
        <Button size="sm" icon={<Copy />} onClick={() => copy(envSnippet)}>
          Copy as environment variables
        </Button>
        <p role="status" className="text-xs text-muted-foreground">
          {result?.kind === "copied" ? "Copied." : null}
        </p>
      </div>
      {result?.kind === "failed" ? <p role="alert">Copying failed.</p> : null}
    </div>
  );
}
