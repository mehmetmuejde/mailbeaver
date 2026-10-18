import { useState } from "react";
import { emlDownloadUrl, type Folder, type MessageSummary } from "@/shared/api";
import { useCopyToClipboard } from "@/shared/lib/clipboard";
import { downloadFile } from "@/shared/lib/platform";
import { useDeleteMessage } from "../api/use-delete-message";
import { useMarkMessageRead } from "../api/use-mark-message-read";
import { useMarkMessageUnread } from "../api/use-mark-message-unread";
import { useMoveMessage } from "../api/use-move-message";
import { senderAddress } from "./sender-address";

/**
 * The actions of the message list's right-click menu, shared by all rows so that one
 * place reports what failed.
 */
export function useMessageActions(onRemoved: (id: number) => void) {
  const markRead = useMarkMessageRead();
  const markUnread = useMarkMessageUnread();
  const move = useMoveMessage();
  const remove = useDeleteMessage();
  const { copy, result: copyResult } = useCopyToClipboard();
  const [downloadFailed, setDownloadFailed] = useState(false);

  const error =
    markRead.isError || markUnread.isError
      ? "The message could not be marked."
      : move.isError
        ? "The message could not be moved."
        : remove.isError
          ? "The message could not be deleted."
          : copyResult?.kind === "failed"
            ? "The address could not be copied."
            : downloadFailed
              ? "The message could not be downloaded."
              : null;

  return {
    error,
    copied: copyResult?.kind === "copied",
    setRead: (message: MessageSummary, read: boolean) =>
      read ? markRead.mutate(message.id) : markUnread.mutate(message.id),
    moveTo: (message: MessageSummary, folder: Folder) =>
      move.mutate({ id: message.id, folder }, { onSuccess: () => onRemoved(message.id) }),
    deleteForGood: (message: MessageSummary) =>
      remove.mutate(message.id, { onSuccess: () => onRemoved(message.id) }),
    copySender: (message: MessageSummary) => copy(senderAddress(message.from)),
    download: (message: MessageSummary) => {
      setDownloadFailed(false);
      downloadFile(emlDownloadUrl(message.id)).catch(() => setDownloadFailed(true));
    },
  };
}

export type MessageActions = ReturnType<typeof useMessageActions>;
