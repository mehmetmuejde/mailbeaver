import { useQuery } from "@tanstack/react-query";
import { Archive, Inbox, Trash2 } from "lucide-react";
import { SmtpSetup } from "@/entities/smtp-server";
import { type Folder, settingsQueries } from "@/shared/api";
import { EmptyState } from "@/shared/ui/empty-state";

type MailEmptyFolderProps = {
  folder: Folder;
};

/** An empty inbox explains how to send mail here; the other folders what they are for. */
export function MailEmptyFolder({ folder }: MailEmptyFolderProps) {
  const { data: info } = useQuery(settingsQueries.info());

  switch (folder) {
    case "inbox":
      return (
        <EmptyState
          icon={<Inbox />}
          title="No messages yet"
          description="Point the SMTP settings of your application at Mailbeaver. Every message it sends lands here instead of a real inbox."
        >
          {info === undefined ? null : <SmtpSetup address={info.smtpAddress} />}
        </EmptyState>
      );
    case "archive":
      return (
        <EmptyState
          icon={<Archive />}
          title="No archived messages"
          description="Archive messages from the inbox to keep them without having them in the way."
        />
      );
    case "trash":
      return (
        <EmptyState
          icon={<Trash2 />}
          title="Trash is empty"
          description="Deleted messages wait here until you empty the trash or restore them."
        />
      );
  }
}
