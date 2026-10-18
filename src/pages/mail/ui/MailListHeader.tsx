import { FolderKanban, List, User } from "lucide-react";
import { SegmentedControl, type SegmentedOption } from "@/shared/ui/segmented-control";
import type { MailSearch, MailSearchChange, MailView } from "../model/mail-search";
import { MailEmptyTrash } from "./MailEmptyTrash";
import { MailFilters } from "./MailFilters";

const VIEW_OPTIONS: readonly SegmentedOption<MailView>[] = [
  { value: "all", label: "All", icon: <List /> },
  { value: "sender", label: "Sender", icon: <User /> },
  { value: "project", label: "Project", icon: <FolderKanban /> },
];

type MailListHeaderProps = {
  search: MailSearch;
  onSearchChange: MailSearchChange;
  /** Shows "Empty trash" below the controls. */
  showEmptyTrash: boolean;
  onTrashEmptied: () => void;
};

/** Above the message list: grouping, filters and, in Trash, emptying it. */
export function MailListHeader({
  search,
  onSearchChange,
  showEmptyTrash,
  onTrashEmptied,
}: MailListHeaderProps) {
  return (
    <div className="grid gap-2 border-b px-3 py-2">
      <div className="flex items-center gap-2">
        <div className="min-w-0 flex-1">
          <SegmentedControl
            label="Group by"
            fullWidth
            options={VIEW_OPTIONS}
            value={search.view ?? "all"}
            onValueChange={(next) =>
              onSearchChange({ view: next === "all" ? undefined : next, page: undefined })
            }
          />
        </div>
        <MailFilters search={search} onSearchChange={onSearchChange} />
      </div>
      {showEmptyTrash ? <MailEmptyTrash onEmptied={onTrashEmptied} /> : null}
    </div>
  );
}
