import { useQuery } from "@tanstack/react-query";
import { Copy } from "lucide-react";
import type { MouseEvent } from "react";
import { settingsQueries } from "@/shared/api";
import { useCopyToClipboard } from "@/shared/lib/clipboard";
import { isDesktopApp, openExternalUrl } from "@/shared/lib/platform";
import { IconButton } from "@/shared/ui/icon-button";

/** How tests, tools and agents such as Claude Code reach Mailbeaver. */
export function SettingsDeveloper() {
  const { data: info } = useQuery(settingsQueries.info());
  const { copy, result } = useCopyToClipboard();
  if (info === undefined) {
    return null;
  }
  const base = `http://${info.httpAddress}`;
  const specUrl = `${base}/api/openapi.json`;
  const mcpCommand = `claude mcp add --transport http mailbeaver ${base}/mcp`;

  // The desktop webview cannot show a download; the system browser opens the file.
  const openSpec = (event: MouseEvent<HTMLAnchorElement>) => {
    if (isDesktopApp()) {
      event.preventDefault();
      void openExternalUrl(specUrl);
    }
  };

  return (
    <section aria-labelledby="developer-heading" className="grid gap-3">
      <h2 id="developer-heading" className="text-base font-semibold">
        API for tools and agents
      </h2>
      <div className="grid gap-1">
        <h3 className="font-medium">OpenAPI</h3>
        <p className="text-muted-foreground">
          Every endpoint the UI uses, e.g. to read mail in end-to-end tests.
        </p>
        <div className="flex items-center gap-2">
          <a
            href={specUrl}
            target="_blank"
            rel="noreferrer"
            onClick={openSpec}
            className="font-mono text-xs text-primary underline underline-offset-2"
          >
            {specUrl}
          </a>
          <IconButton label="Copy OpenAPI URL" icon={<Copy />} onClick={() => copy(specUrl)} />
        </div>
      </div>
      <div className="grid gap-1">
        <h3 className="font-medium">Mailpit-compatible API</h3>
        <p className="text-muted-foreground">
          Test helpers written for Mailpit work when pointed at{" "}
          <code className="font-mono text-xs">{base}</code>; they find the endpoints under{" "}
          <code className="font-mono text-xs">/api/v1</code>.
        </p>
      </div>
      <div className="grid gap-1">
        <h3 className="font-medium">MCP server</h3>
        <p className="text-muted-foreground">
          Lets Claude Code and other agents list, wait for and read mail. It cannot delete anything.
        </p>
        <div className="flex items-center gap-2">
          <code className="rounded bg-muted px-2 py-1 font-mono text-xs break-all">
            {mcpCommand}
          </code>
          <IconButton label="Copy MCP command" icon={<Copy />} onClick={() => copy(mcpCommand)} />
        </div>
      </div>
      <p role="status" className="text-xs text-muted-foreground">
        {result?.kind === "copied" ? "Copied." : null}
      </p>
      {result?.kind === "failed" ? <p role="alert">Copying failed.</p> : null}
    </section>
  );
}
