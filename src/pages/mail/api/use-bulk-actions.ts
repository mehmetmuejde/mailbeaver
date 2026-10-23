import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  deleteMessagePermanently,
  type Folder,
  markMessageRead,
  markMessageUnread,
  messageKeys,
  moveMessage,
} from "@/shared/api";

export type BulkAction =
  | { kind: "move"; folder: Folder }
  | { kind: "read" }
  | { kind: "unread" }
  | { kind: "delete" };

function run(action: BulkAction, id: number): Promise<void> {
  switch (action.kind) {
    case "move":
      return moveMessage({ id, folder: action.folder });
    case "read":
      return markMessageRead(id);
    case "unread":
      return markMessageUnread(id);
    case "delete":
      return deleteMessagePermanently(id);
  }
}

/** One action for many messages; the lists refresh once at the end. */
export function useBulkActions() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ ids, action }: { ids: readonly number[]; action: BulkAction }) => {
      await Promise.all(ids.map((id) => run(action, id)));
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
