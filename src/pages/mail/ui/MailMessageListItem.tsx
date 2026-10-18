import { Paperclip } from "lucide-react";
import type { ComponentProps } from "react";
import type { MessageSummary } from "@/shared/api";
import { FOLDER_LABELS } from "@/shared/config";
import { formatBytes, formatDateTime, formatRelativeTime, toIsoString } from "@/shared/lib/format";
import { UNKNOWN_SENDER } from "../model/group-messages";

/** Extra button props (handlers, ref) come from wrappers such as the context menu. */
type MailMessageListItemProps = Omit<ComponentProps<"button">, "onSelect" | "children"> & {
  message: MessageSummary;
  isSelected: boolean;
  showProject: boolean;
  /** Search results come from every folder; each one says where it lives. */
  showFolder?: boolean;
  onSelect: (message: MessageSummary) => void;
};

function recipients(to: readonly string[]): string {
  const [first, ...rest] = to;
  if (first === undefined) {
    return "(no recipient)";
  }
  return rest.length > 0 ? `${first} +${rest.length}` : first;
}

export function MailMessageListItem({
  message,
  isSelected,
  showProject,
  showFolder = false,
  onSelect,
  ...buttonProps
}: MailMessageListItemProps) {
  const unread = !message.isRead;
  return (
    <button
      {...buttonProps}
      type="button"
      aria-current={isSelected ? "true" : undefined}
      onClick={() => onSelect(message)}
      className="grid w-full gap-0.5 border-b px-4 py-3 text-left hover:bg-muted aria-[current=true]:bg-accent aria-[current=true]:shadow-[inset_3px_0_0_var(--primary)]"
    >
      <span className="flex items-baseline gap-2">
        <span className={`flex min-w-0 flex-1 items-center gap-2 ${unread ? "font-semibold" : ""}`}>
          {unread ? (
            <>
              <span aria-hidden="true" className="size-2 shrink-0 rounded-full bg-unread" />
              <span className="sr-only">Unread: </span>
            </>
          ) : null}
          <span className="truncate">{message.from || UNKNOWN_SENDER}</span>
        </span>
        <time
          dateTime={toIsoString(message.receivedAt)}
          title={formatDateTime(message.receivedAt)}
          className="shrink-0 text-xs text-muted-foreground"
        >
          {formatRelativeTime(message.receivedAt)}
        </time>
      </span>
      <span className={`truncate ${unread ? "font-semibold" : ""}`}>
        {message.subject || "(no subject)"}
      </span>
      {message.preview === "" ? null : (
        <span className="truncate text-xs text-muted-foreground">{message.preview}</span>
      )}
      <span className="flex min-w-0 items-center gap-1 text-xs text-muted-foreground">
        {showFolder ? (
          <span className="shrink-0 rounded bg-muted px-1.5 font-medium text-foreground">
            {FOLDER_LABELS[message.folder]}
          </span>
        ) : null}
        <span className="truncate">To: {recipients(message.to)}</span>
        <span className="shrink-0">{` · ${formatBytes(message.size)}`}</span>
        {showProject && message.project !== null ? (
          <span className="shrink-0">{` · ${message.project.name}`}</span>
        ) : null}
        {message.hasAttachments ? (
          <>
            {" · "}
            <Paperclip aria-hidden="true" className="size-3 shrink-0" />
            <span className="sr-only">Attachment</span>
          </>
        ) : null}
      </span>
    </button>
  );
}
