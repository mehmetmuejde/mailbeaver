import { CircleHelp } from "lucide-react";
import { IconButton } from "@/shared/ui/icon-button";
import { Popover } from "@/shared/ui/popover";

/** The filters the server understands in the search field (ADR-0027). */
const FILTERS = [
  { example: "from:shop.test", meaning: "Sender" },
  { example: "to:dev@team.test", meaning: "Recipient" },
  { example: 'subject:"order 12"', meaning: "Subject" },
  { example: "body:reset", meaning: "Body text" },
  { example: "is:unread", meaning: "Unread (or is:read)" },
  { example: "has:attachment", meaning: "With attachments" },
  { example: "project:shop", meaning: "Project (or project:none)" },
  { example: "after:2026-10-01", meaning: "Received on or after (UTC)" },
  { example: "before:2026-10-17", meaning: "Received before (UTC)" },
] as const;

export function MailSearchHelp() {
  return (
    <Popover
      label="Search syntax"
      width="wide"
      trigger={<IconButton label="Search syntax" icon={<CircleHelp />} />}
    >
      <p className="text-sm">
        Every word must match. Combine words with filters, e.g.{" "}
        <code className="font-mono text-xs">from:shop is:unread invoice</code>.
      </p>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
        {FILTERS.map((filter) => (
          <div key={filter.example} className="contents">
            <dt>
              <code className="font-mono text-xs">{filter.example}</code>
            </dt>
            <dd className="text-muted-foreground">{filter.meaning}</dd>
          </div>
        ))}
      </dl>
    </Popover>
  );
}
