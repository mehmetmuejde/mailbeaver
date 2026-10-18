import type { MouseEvent } from "react";
import { useState } from "react";
import { type AttachmentInfo, attachmentUrl } from "@/shared/api";
import { formatBytes } from "@/shared/lib/format";
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
    <section aria-labelledby="attachments-heading">
      <h3 id="attachments-heading" className="font-semibold">
        Attachments
      </h3>
      <ul className="mt-1 list-disc pl-5">
        {attachments.map((attachment) => {
          const url = attachmentUrl(messageId, attachment.index);
          return (
            <li key={attachment.index}>
              <a
                href={url}
                download={attachment.filename}
                className="text-primary underline underline-offset-2"
                onClick={(e) => handleClick(e, url)}
              >
                {attachment.filename}
              </a>{" "}
              <span className="text-xs text-muted-foreground">
                {attachment.contentType}, {formatBytes(attachment.size)}
              </span>
            </li>
          );
        })}
      </ul>
      {openError === null ? null : <p role="alert">{openError}</p>}
    </section>
  );
}
