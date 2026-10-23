import { Archive, AtSign, Download, Inbox, Mail, MailOpen, Trash2, Undo2 } from "lucide-react";
import type { ReactNode } from "react";
import type { MessageSummary } from "@/shared/api";
import { ContextMenu, type ContextMenuGroups } from "@/shared/ui/context-menu";
import type { BulkAction } from "../api/use-bulk-actions";
import type { MessageActions } from "../model/use-message-actions";

type MailMessageContextMenuProps = {
  message: MessageSummary;
  actions: MessageActions;
  /** The selected messages when several are selected and this is one of them. */
  selected: readonly MessageSummary[];
  onBulk: (action: BulkAction) => void;
  children: ReactNode;
};

/** The same actions as for one message, applied to every selected message. */
function bulkGroups(
  selected: readonly MessageSummary[],
  onBulk: (action: BulkAction) => void,
): ContextMenuGroups {
  const count = ` (${selected.length})`;
  const allInTrash = selected.every((message) => message.folder === "trash");
  const anyInInbox = selected.some((message) => message.folder === "inbox");
  return [
    [
      {
        label: `Mark as read${count}`,
        icon: <MailOpen />,
        onSelect: () => onBulk({ kind: "read" }),
      },
      {
        label: `Mark as unread${count}`,
        icon: <Mail />,
        onSelect: () => onBulk({ kind: "unread" }),
      },
    ],
    allInTrash
      ? [
          {
            label: `Restore to inbox${count}`,
            icon: <Undo2 />,
            onSelect: () => onBulk({ kind: "move", folder: "inbox" }),
          },
          {
            label: `Delete permanently${count}`,
            icon: <Trash2 />,
            destructive: true,
            onSelect: () => onBulk({ kind: "delete" }),
          },
        ]
      : [
          anyInInbox
            ? {
                label: `Archive${count}`,
                icon: <Archive />,
                onSelect: () => onBulk({ kind: "move", folder: "archive" }),
              }
            : {
                label: `Move to inbox${count}`,
                icon: <Inbox />,
                onSelect: () => onBulk({ kind: "move", folder: "inbox" }),
              },
          {
            label: `Move to trash${count}`,
            icon: <Trash2 />,
            onSelect: () => onBulk({ kind: "move", folder: "trash" }),
          },
        ],
  ];
}

/** Right-click menu of a message; the folder decides which moves are offered. */
export function MailMessageContextMenu({
  message,
  actions,
  selected,
  onBulk,
  children,
}: MailMessageContextMenuProps) {
  if (selected.length > 1) {
    return <ContextMenu groups={bulkGroups(selected, onBulk)}>{children}</ContextMenu>;
  }
  const folderActions =
    message.folder === "trash"
      ? [
          {
            label: "Restore to inbox",
            icon: <Undo2 />,
            onSelect: () => actions.moveTo(message, "inbox"),
          },
          {
            label: "Delete permanently",
            icon: <Trash2 />,
            destructive: true,
            onSelect: () => actions.deleteForGood(message),
          },
        ]
      : [
          message.folder === "inbox"
            ? {
                label: "Archive",
                icon: <Archive />,
                onSelect: () => actions.moveTo(message, "archive"),
              }
            : {
                label: "Move to inbox",
                icon: <Inbox />,
                onSelect: () => actions.moveTo(message, "inbox"),
              },
          {
            label: "Move to trash",
            icon: <Trash2 />,
            onSelect: () => actions.moveTo(message, "trash"),
          },
        ];

  const groups: ContextMenuGroups = [
    [
      message.isRead
        ? {
            label: "Mark as unread",
            icon: <Mail />,
            onSelect: () => actions.setRead(message, false),
          }
        : {
            label: "Mark as read",
            icon: <MailOpen />,
            onSelect: () => actions.setRead(message, true),
          },
    ],
    folderActions,
    [
      ...(message.from === ""
        ? []
        : [
            {
              label: "Copy sender address",
              icon: <AtSign />,
              onSelect: () => actions.copySender(message),
            },
          ]),
      {
        label: "Download as .eml",
        icon: <Download />,
        onSelect: () => actions.download(message),
      },
    ],
  ];

  return <ContextMenu groups={groups}>{children}</ContextMenu>;
}
