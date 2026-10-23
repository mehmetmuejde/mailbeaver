import { Archive, Inbox, Mail, MailOpen, Trash2, Undo2, X } from "lucide-react";
import type { MessageSummary } from "@/shared/api";
import { pluralize } from "@/shared/lib/format";
import { Button } from "@/shared/ui/button";
import { type BulkAction, useBulkActions } from "../api/use-bulk-actions";

type MailSelectionPanelProps = {
  messages: readonly MessageSummary[];
  onClear: () => void;
};

/** Shown instead of a message while several are selected: actions for all of them. */
export function MailSelectionPanel({ messages, onClear }: MailSelectionPanelProps) {
  const bulk = useBulkActions();
  const ids = messages.map((message) => message.id);
  const allInTrash = messages.every((message) => message.folder === "trash");
  const anyInInbox = messages.some((message) => message.folder === "inbox");
  const apply = (action: BulkAction, clears: boolean) =>
    bulk.mutate({ ids, action }, { onSuccess: clears ? onClear : undefined });

  return (
    <section aria-labelledby="selection-heading" className="m-auto grid justify-items-center gap-4">
      <h2 id="selection-heading" className="text-lg font-semibold">
        {pluralize(messages.length, "message")} selected
      </h2>
      <div className="flex max-w-md flex-wrap justify-center gap-2">
        <Button
          icon={<MailOpen />}
          disabled={bulk.isPending}
          onClick={() => apply({ kind: "read" }, false)}
        >
          Mark as read
        </Button>
        <Button
          icon={<Mail />}
          disabled={bulk.isPending}
          onClick={() => apply({ kind: "unread" }, false)}
        >
          Mark as unread
        </Button>
        {allInTrash ? (
          <>
            <Button
              icon={<Undo2 />}
              disabled={bulk.isPending}
              onClick={() => apply({ kind: "move", folder: "inbox" }, true)}
            >
              Restore to inbox
            </Button>
            <Button
              variant="destructive"
              icon={<Trash2 />}
              disabled={bulk.isPending}
              onClick={() => apply({ kind: "delete" }, true)}
            >
              Delete permanently
            </Button>
          </>
        ) : (
          <>
            {anyInInbox ? (
              <Button
                icon={<Archive />}
                disabled={bulk.isPending}
                onClick={() => apply({ kind: "move", folder: "archive" }, true)}
              >
                Archive
              </Button>
            ) : (
              <Button
                icon={<Inbox />}
                disabled={bulk.isPending}
                onClick={() => apply({ kind: "move", folder: "inbox" }, true)}
              >
                Move to inbox
              </Button>
            )}
            <Button
              icon={<Trash2 />}
              disabled={bulk.isPending}
              onClick={() => apply({ kind: "move", folder: "trash" }, true)}
            >
              Move to trash
            </Button>
          </>
        )}
        <Button variant="ghost" icon={<X />} onClick={onClear}>
          Clear selection
        </Button>
      </div>
      {bulk.isError ? <p role="alert">Some messages could not be changed.</p> : null}
    </section>
  );
}
