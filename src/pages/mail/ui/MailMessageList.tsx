import { useQuery } from "@tanstack/react-query";
import { RotateCw } from "lucide-react";
import { type KeyboardEvent, type MouseEvent, useEffect, useRef } from "react";
import { type MessageListParams, type MessageSummary, messageQueries } from "@/shared/api";
import { isMacPlatform } from "@/shared/lib/platform";
import { Button } from "@/shared/ui/button";
import { useBulkActions } from "../api/use-bulk-actions";
import { groupMessages } from "../model/group-messages";
import type { MailView } from "../model/mail-search";
import {
  clickSelection,
  EMPTY_SELECTION,
  isToggleClick,
  rangeBetween,
  type Selection,
} from "../model/selection";
import { useMessageActions } from "../model/use-message-actions";
import { MailMessageContextMenu } from "./MailMessageContextMenu";
import { MailMessageListItem } from "./MailMessageListItem";

type MailMessageListProps = {
  params: MessageListParams;
  view: MailView;
  /** Search results come from every folder and show where each message lives. */
  showFolder?: boolean;
  selectedId: number | null;
  /** Several selected messages; one selected message is `selectedId`. */
  selection: Selection;
  onSelectionChange: (selection: Selection) => void;
  /** A plain click: opens the message. */
  onSelect: (message: MessageSummary) => void;
  /** Called after a message left the list through its context menu. */
  onRemoved: (id: number) => void;
};

export function MailMessageList({
  params,
  view,
  showFolder = false,
  selectedId,
  selection,
  onSelectionChange,
  onSelect,
  onRemoved,
}: MailMessageListProps) {
  const { data, isPending, isError, refetch } = useQuery(messageQueries.list(params));
  const actions = useMessageActions(onRemoved);
  const bulk = useBulkActions();
  // Where a drag over the list started; a drag selects every message it passes.
  const dragStart = useRef<number | null>(null);

  useEffect(() => {
    const endDrag = () => {
      dragStart.current = null;
    };
    window.addEventListener("pointerup", endDrag);
    return () => window.removeEventListener("pointerup", endDrag);
  }, []);

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

  const groups = groupMessages(data.messages, view);
  // The visible order, which grouping may change: ranges follow what the user sees.
  const order = groups.flatMap((group) => group.messages.map((message) => message.id));
  const isMulti = selection.ids.size > 1;
  const selectedMessages = isMulti
    ? data.messages.filter((message) => selection.ids.has(message.id))
    : [];

  const handleClick = (message: MessageSummary, event: MouseEvent<HTMLButtonElement>) => {
    const modifiers = { toggle: isToggleClick(event, isMacPlatform()), range: event.shiftKey };
    if (!modifiers.toggle && !modifiers.range) {
      onSelect(message);
      return;
    }
    // Without a multi-selection the open message is where a range or toggle starts.
    const current =
      isMulti || selectedId === null
        ? selection
        : { ids: new Set([selectedId]), anchor: selectedId };
    onSelectionChange(clickSelection(current, message.id, order, modifiers));
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "a" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      onSelectionChange({ ids: new Set(order), anchor: order[0] ?? null });
    } else if (event.key === "Escape" && isMulti) {
      onSelectionChange(EMPTY_SELECTION);
    }
  };

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: keyboard shortcuts of the focused list items bubble here.
    <div onKeyDown={handleKeyDown}>
      {actions.error === null ? null : (
        <p role="alert" className="border-b px-4 py-2 text-destructive">
          {actions.error}
        </p>
      )}
      <p role="status" className="sr-only">
        {actions.copied ? "Address copied." : null}
      </p>
      {bulk.isError ? (
        <p role="alert" className="border-b px-4 py-2 text-destructive">
          Some messages could not be changed.
        </p>
      ) : null}
      {groups.map((group) => (
        <section key={group.key} aria-label={group.label ?? undefined}>
          {group.label === null ? null : (
            <h2 className="sticky top-0 z-10 truncate border-b bg-muted px-4 py-1 text-xs font-semibold text-muted-foreground">
              {group.label}
            </h2>
          )}
          <ul>
            {group.messages.map((message) => (
              <li key={message.id}>
                <MailMessageContextMenu
                  message={message}
                  actions={actions}
                  selected={selection.ids.has(message.id) ? selectedMessages : []}
                  onBulk={(action) =>
                    bulk.mutate(
                      { ids: selectedMessages.map((selected) => selected.id), action },
                      {
                        onSuccess: () => {
                          if (action.kind === "move" || action.kind === "delete") {
                            onSelectionChange(EMPTY_SELECTION);
                          }
                        },
                      },
                    )
                  }
                >
                  <MailMessageListItem
                    message={message}
                    isSelected={!isMulti && message.id === selectedId}
                    isChecked={isMulti && selection.ids.has(message.id)}
                    showProject={view !== "project"}
                    showFolder={showFolder}
                    onSelect={handleClick}
                    onPointerDown={(event) => {
                      if (
                        event.button === 0 &&
                        !event.shiftKey &&
                        !event.metaKey &&
                        !event.ctrlKey
                      ) {
                        dragStart.current = message.id;
                      }
                    }}
                    onPointerEnter={(event) => {
                      const start = dragStart.current;
                      if (start !== null && start !== message.id && event.buttons === 1) {
                        onSelectionChange({
                          ids: new Set(rangeBetween(order, start, message.id)),
                          anchor: start,
                        });
                      }
                    }}
                  />
                </MailMessageContextMenu>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
