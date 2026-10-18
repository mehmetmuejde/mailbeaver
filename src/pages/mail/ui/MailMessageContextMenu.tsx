import { Archive, AtSign, Download, Inbox, Mail, MailOpen, Trash2, Undo2 } from "lucide-react";
import type { ReactNode } from "react";
import type { MessageSummary } from "@/shared/api";
import { ContextMenu, type ContextMenuGroups } from "@/shared/ui/context-menu";
import type { MessageActions } from "../model/use-message-actions";

type MailMessageContextMenuProps = {
  message: MessageSummary;
  actions: MessageActions;
  children: ReactNode;
};

/** Right-click menu of a message; the folder decides which moves are offered. */
export function MailMessageContextMenu({
  message,
  actions,
  children,
}: MailMessageContextMenuProps) {
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
