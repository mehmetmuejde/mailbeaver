import { useQuery } from "@tanstack/react-query";
import { RotateCw } from "lucide-react";
import { type MessageListParams, type MessageSummary, messageQueries } from "@/shared/api";
import { Button } from "@/shared/ui/button";
import { groupMessages } from "../model/group-messages";
import type { MailView } from "../model/mail-search";
import { useMessageActions } from "../model/use-message-actions";
import { MailMessageContextMenu } from "./MailMessageContextMenu";
import { MailMessageListItem } from "./MailMessageListItem";

type MailMessageListProps = {
  params: MessageListParams;
  view: MailView;
  /** Search results come from every folder and show where each message lives. */
  showFolder?: boolean;
  selectedId: number | null;
  onSelect: (message: MessageSummary) => void;
  /** Called after a message left the list through its context menu. */
  onRemoved: (id: number) => void;
};

export function MailMessageList({
  params,
  view,
  showFolder = false,
  selectedId,
  onSelect,
  onRemoved,
}: MailMessageListProps) {
  const { data, isPending, isError, refetch } = useQuery(messageQueries.list(params));
  const actions = useMessageActions(onRemoved);

  if (isPending) {
    return (
      <p role="status" className="p-4 text-muted-foreground">
        Loading messages…
      </p>
    );
  }
  if (isError) {
    return (
      <div role="alert" className="grid justify-items-start gap-2 p-4">
        <p>Messages could not be loaded.</p>
        <Button icon={<RotateCw />} onClick={() => void refetch()}>
          Retry
        </Button>
      </div>
    );
  }
  // An empty folder without filters is shown by the page; here only filters can empty it.
  if (data.messages.length === 0) {
    const query = params.q?.trim() ?? "";
    return (
      <p className="p-4 text-muted-foreground">
        {query === "" ? "No messages in this project." : `No messages match “${query}”.`}
      </p>
    );
  }

  return (
    <>
      {actions.error === null ? null : (
        <p role="alert" className="border-b px-4 py-2 text-destructive">
          {actions.error}
        </p>
      )}
      <p role="status" className="sr-only">
        {actions.copied ? "Address copied." : null}
      </p>
      {groupMessages(data.messages, view).map((group) => (
        <section key={group.key} aria-label={group.label ?? undefined}>
          {group.label === null ? null : (
            <h2 className="sticky top-0 z-10 truncate border-b bg-muted px-4 py-1 text-xs font-semibold text-muted-foreground">
              {group.label}
            </h2>
          )}
          <ul>
            {group.messages.map((message) => (
              <li key={message.id}>
                <MailMessageContextMenu message={message} actions={actions}>
                  <MailMessageListItem
                    message={message}
                    isSelected={message.id === selectedId}
                    showProject={view !== "project"}
                    showFolder={showFolder}
                    onSelect={onSelect}
                  />
                </MailMessageContextMenu>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </>
  );
}
