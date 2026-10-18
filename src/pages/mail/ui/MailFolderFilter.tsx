import { Archive, Inbox, Layers, Trash2 } from "lucide-react";
import { FOLDERS, type Folder } from "@/shared/api";
import { FOLDER_LABELS } from "@/shared/config";
import { SegmentedControl, type SegmentedOption } from "@/shared/ui/segmented-control";

type FolderChoice = Folder | "all";

const FOLDER_ICONS = { inbox: <Inbox />, archive: <Archive />, trash: <Trash2 /> } as const;

const OPTIONS: readonly SegmentedOption<FolderChoice>[] = [
  { value: "all", label: "All folders", icon: <Layers /> },
  ...FOLDERS.map((folder) => ({
    value: folder,
    label: FOLDER_LABELS[folder],
    icon: FOLDER_ICONS[folder],
  })),
];

type MailFolderFilterProps = {
  /** `undefined` searches every folder. */
  folder: Folder | undefined;
  onChange: (folder: Folder | undefined) => void;
};

/** Narrows search results to one folder. */
export function MailFolderFilter({ folder, onChange }: MailFolderFilterProps) {
  return (
    <SegmentedControl
      label="Search in"
      options={OPTIONS}
      value={folder ?? "all"}
      onValueChange={(choice) => onChange(choice === "all" ? undefined : choice)}
    />
  );
}
