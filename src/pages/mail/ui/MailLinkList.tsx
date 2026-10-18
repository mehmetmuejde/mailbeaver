import { Copy, SearchCheck } from "lucide-react";
import { useId } from "react";
import { useCopyToClipboard } from "@/shared/lib/clipboard";
import { Button } from "@/shared/ui/button";
import { IconButton } from "@/shared/ui/icon-button";
import { useCheckLinks } from "../api/use-check-links";
import { extractLinks } from "../lib/extract-links";
import { MailLinkStatus } from "./MailLinkStatus";

type MailLinkListProps = {
  html: string | null;
  text: string | null;
};

/** Links of the message to check or copy, e.g. confirmation or password reset links. */
export function MailLinkList({ html, text }: MailLinkListProps) {
  const baseId = useId();
  const { copy, result } = useCopyToClipboard();
  const links = extractLinks(html, text);
  const check = useCheckLinks();
  // mailto: links have no HTTP status.
  const webLinks = links.filter((link) => /^https?:/.test(link.url)).map((link) => link.url);
  const statusByUrl = new Map(check.data?.map((status) => [status.url, status]));

  if (links.length === 0) {
    return <p className="text-muted-foreground">This message contains no links.</p>;
  }

  return (
    <div className="grid gap-2">
      {webLinks.length === 0 ? null : (
        <div>
          <Button
            size="sm"
            icon={<SearchCheck />}
            disabled={check.isPending}
            onClick={() => check.mutate(webLinks)}
          >
            {check.isPending ? "Checking links…" : "Check links"}
          </Button>
        </div>
      )}
      {check.isError ? <p role="alert">The links could not be checked.</p> : null}
      <ul className="divide-y">
        {links.map((link, index) => {
          const urlId = `${baseId}-${index}`;
          return (
            <li key={link.url} className="flex items-center gap-2 py-1.5">
              <div className="min-w-0 flex-1">
                {link.text === null ? null : <p className="truncate">{link.text}</p>}
                <code
                  id={urlId}
                  className="block font-mono text-xs break-all text-muted-foreground"
                >
                  {link.url}
                </code>
              </div>
              <MailLinkStatus status={statusByUrl.get(link.url)} />
              <IconButton
                label="Copy link"
                icon={<Copy />}
                aria-describedby={urlId}
                onClick={() => copy(link.url)}
              />
            </li>
          );
        })}
      </ul>
      <p role="status" className="text-muted-foreground">
        {result?.kind === "copied" ? "Link copied." : null}
      </p>
      {result?.kind === "failed" ? <p role="alert">The link could not be copied.</p> : null}
    </div>
  );
}
