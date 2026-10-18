import { Archive, Download, Inbox, Trash2, Undo2 } from "lucide-react";
import { useState } from "react";
import { emlDownloadUrl, type Folder, type MessageDetail } from "@/shared/api";
import { downloadFile } from "@/shared/lib/platform";
import { IconButton } from "@/shared/ui/icon-button";
import { useDeleteMessage } from "../api/use-delete-message";
import { useMoveMessage } from "../api/use-move-message";

type MailMessageActionsProps = {
  message: MessageDetail;
  onDone: () => void;
};

/**
 * Moving never asks: Trash is the undo. Only a message already in Trash can be deleted
 * for good, which also needs no confirmation.
 */
export function MailMessageActions({ message, onDone }: MailMessageActionsProps) {
  const move = useMoveMessage();
  const remove = useDeleteMessage();
  const isBusy = move.isPending || remove.isPending;
  const [downloadFailed, setDownloadFailed] = useState(false);

  const moveTo = (folder: Folder) => move.mutate({ id: message.id, folder }, { onSuccess: onDone });

  return (
    <div className="flex shrink-0 flex-col items-end gap-1">
      <div className="flex gap-1">
        <IconButton
          label="Download as .eml"
          icon={<Download />}
          onClick={() => {
            setDownloadFailed(false);
            downloadFile(emlDownloadUrl(message.id)).catch(() => setDownloadFailed(true));
          }}
        />
        {message.folder === "inbox" ? (
          <IconButton
            label="Archive"
            icon={<Archive />}
            disabled={isBusy}
            onClick={() => moveTo("archive")}
          />
        ) : null}
        {message.folder === "archive" ? (
          <IconButton
            label="Move to inbox"
            icon={<Inbox />}
            disabled={isBusy}
            onClick={() => moveTo("inbox")}
          />
        ) : null}
        {message.folder === "trash" ? (
          <>
            <IconButton
              label="Restore to inbox"
              icon={<Undo2 />}
              disabled={isBusy}
              onClick={() => moveTo("inbox")}
            />
            <IconButton
              label="Delete permanently"
              icon={<Trash2 />}
              variant="destructive"
              disabled={isBusy}
              onClick={() => remove.mutate(message.id, { onSuccess: onDone })}
            />
          </>
        ) : (
          <IconButton
            label="Move to trash"
            icon={<Trash2 />}
            disabled={isBusy}
            onClick={() => moveTo("trash")}
          />
        )}
      </div>
      {move.isError ? <p role="alert">The message could not be moved.</p> : null}
      {remove.isError ? <p role="alert">The message could not be deleted.</p> : null}
      {downloadFailed ? <p role="alert">The message could not be downloaded.</p> : null}
    </div>
  );
}
