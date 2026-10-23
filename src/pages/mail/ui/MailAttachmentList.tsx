import { Download, FileText, Paperclip } from "lucide-react";
import type { MouseEvent } from "react";
import { useState } from "react";
import { type AttachmentInfo, attachmentUrl } from "@/shared/api";
import { formatBytes, pluralize } from "@/shared/lib/format";
import { isDesktopApp, openExternalUrl } from "@/shared/lib/platform";

type MailAttachmentListProps = {
  messageId: number;
  attachments: readonly AttachmentInfo[];
};

export function MailAttachmentList({ messageId, attachments }: MailAttachmentListProps) {
  const [openError, setOpenError] = useState<string | null>(null);

  // The desktop webview has no download manager; the system browser saves the file.
  const handleClick = (event: MouseEvent<HTMLAnchorElement>, url: string) => {
    if (!isDesktopApp()) {
      return;
    }
    event.preventDefault();
    setOpenError(null);
    openExternalUrl(url).catch(() => setOpenError("The attachment could not be opened."));
  };

  return (
    <section
      aria-labelledby="attachments-heading"
      className="grid shrink-0 gap-2 rounded-lg border border-amber-300 bg-amber-50 p-3 dark:border-amber-700/60 dark:bg-amber-950/40"
    >
      <h3
        id="attachments-heading"
        className="flex items-center gap-2 font-semibold text-amber-950 dark:text-amber-100"
      >
        <Paperclip aria-hidden="true" className="size-4" />
        {pluralize(attachments.length, "attachment")}
      </h3>
      <ul className="grid max-h-48 gap-1 overflow-y-auto">
        {attachments.map((attachment) => {
          const url = attachmentUrl(messageId, attachment.index);
          return (
            <li
              key={attachment.index}
              className="flex items-center gap-3 rounded-md bg-background/70 px-3 py-2"
            >
              <FileText
                aria-hidden="true"
                className="size-4 shrink-0 text-amber-700 dark:text-amber-300"
              />
              <a
                href={url}
                download={attachment.filename}
                className="min-w-0 flex-1 truncate font-medium text-primary underline-offset-2 hover:underline"
                onClick={(e) => handleClick(e, url)}
              >
                {attachment.filename}
              </a>
              <span className="shrink-0 text-xs text-muted-foreground">
                {attachment.contentType}, {formatBytes(attachment.size)}
              </span>
              <Download aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
            </li>
          );
        })}
      </ul>
      {openError === null ? null : <p role="alert">{openError}</p>}
    </section>
  );
}
