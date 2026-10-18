import { useQuery } from "@tanstack/react-query";
import { Archive, Inbox, Settings, Trash2 } from "lucide-react";
import { type Folder, messageQueries } from "@/shared/api";
import { FOLDER_LABELS } from "@/shared/config";
import { AppNavItem } from "./AppNavItem";
import { useMarkFolderRead } from "./use-mark-folder-read";

export function AppNavigation() {
  const { data: counts } = useQuery(messageQueries.folders());
  const markRead = useMarkFolderRead();
  const markAllRead = (folder: Folder) => () => markRead.mutate(folder);

  return (
    <nav aria-label="Folders" className="px-3">
      <ul className="grid gap-1">
        <AppNavItem
          to="/inbox"
          icon={<Inbox />}
          label={FOLDER_LABELS.inbox}
          counts={counts?.inbox}
          onMarkAllRead={markAllRead("inbox")}
        />
        <AppNavItem
          to="/archive"
          icon={<Archive />}
          label={FOLDER_LABELS.archive}
          counts={counts?.archive}
          onMarkAllRead={markAllRead("archive")}
        />
        <AppNavItem
          to="/trash"
          icon={<Trash2 />}
          label={FOLDER_LABELS.trash}
          counts={counts?.trash}
          onMarkAllRead={markAllRead("trash")}
        />
        <AppNavItem to="/settings" icon={<Settings />} label="Settings" />
      </ul>
      {markRead.isError ? (
        <p role="alert" className="px-3 py-2 text-destructive">
          Messages could not be marked read.
        </p>
      ) : null}
    </nav>
  );
}
