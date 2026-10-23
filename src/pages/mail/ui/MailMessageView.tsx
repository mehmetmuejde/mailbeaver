import { useQuery } from "@tanstack/react-query";
import { isNotFound, messageQueries } from "@/shared/api";
import { formatDateTime, toIsoString } from "@/shared/lib/format";
import { UNKNOWN_SENDER } from "../model/group-messages";
import { MailAttachmentList } from "./MailAttachmentList";
import { MailMessageActions } from "./MailMessageActions";
import { MailMessageBody } from "./MailMessageBody";

type MailMessageViewProps = {
  messageId: number;
  /** Called once the message has left the current view (moved or deleted). */
  onClose: () => void;
};

export function MailMessageView({ messageId, onClose }: MailMessageViewProps) {
  const { data: message, isPending, error } = useQuery(messageQueries.detail(messageId));

  if (isPending) {
    return <p role="status">Loading message…</p>;
  }
  if (error !== null) {
    return (
      <p role="alert">
        {isNotFound(error) ? "This message no longer exists." : "The message could not be loaded."}
      </p>
    );
  }

  const recipients = message.to.length > 0 ? message.to : message.envelopeTo;
  return (
    <article aria-labelledby="message-subject" className="flex min-h-0 flex-1 flex-col gap-3">
      <header className="flex items-start justify-between gap-4">
        <h2 id="message-subject" className="text-base font-semibold break-words">
          {message.subject || "(no subject)"}
        </h2>
        <MailMessageActions message={message} onDone={onClose} />
      </header>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-1">
        <dt className="text-muted-foreground">From</dt>
        <dd className="break-all">{message.from || message.envelopeFrom || UNKNOWN_SENDER}</dd>
        <dt className="text-muted-foreground">To</dt>
        <dd className="break-all">{recipients.join(", ")}</dd>
        <dt className="text-muted-foreground">Received</dt>
        <dd>
          <time dateTime={toIsoString(message.receivedAt)}>
            {formatDateTime(message.receivedAt)}
          </time>
        </dd>
        {message.project === null ? null : (
          <>
            <dt className="text-muted-foreground">Project</dt>
            <dd>{message.project.name}</dd>
          </>
        )}
        {message.authUser === null ? null : (
          <>
            <dt className="text-muted-foreground">SMTP user</dt>
            <dd>{message.authUser}</dd>
          </>
        )}
      </dl>
      <MailMessageBody message={message} />
      {/* Below the message, where a reader looks after reading it. */}
      {message.attachments.length > 0 ? (
        <MailAttachmentList messageId={message.id} attachments={message.attachments} />
      ) : null}
    </article>
  );
}
