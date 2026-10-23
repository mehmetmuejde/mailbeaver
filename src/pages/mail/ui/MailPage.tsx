import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import {
  type Folder,
  type MessageListParams,
  type MessageSummary,
  messageQueries,
} from "@/shared/api";
import { APP_NAME, FOLDER_LABELS } from "@/shared/config";
import { PageHeader } from "@/shared/ui/page-header";
import { useMarkMessageRead } from "../api/use-mark-message-read";
import { useSaveFolderView } from "../api/use-save-folder-view";
import { searchToView, VIEW_KEYS } from "../model/folder-view";
import { DEFAULT_PAGE_SIZE, type MailSearch, type MailSearchChange } from "../model/mail-search";
import { periodStart } from "../model/period";
import { EMPTY_SELECTION, keepVisible, type Selection, selectOne } from "../model/selection";
import { MailEmptyFolder } from "./MailEmptyFolder";
import { MailFolderFilter } from "./MailFolderFilter";
import { MailListHeader } from "./MailListHeader";
import { MailMessageList } from "./MailMessageList";
import { MailMessageView } from "./MailMessageView";
import { MailPagination } from "./MailPagination";
import { MailSearchBox } from "./MailSearchBox";
import { MailSelectionPanel } from "./MailSelectionPanel";
import { MailStatus } from "./MailStatus";

type MailPageProps = {
  folder: Folder;
  search: MailSearch;
  onSearchChange: MailSearchChange;
};

/**
 * A folder view. Typing in the search field turns it into results from every folder;
 * all view state comes from the URL.
 */
export function MailPage({ folder, search, onSearchChange: navigate }: MailPageProps) {
  const saveView = useSaveFolderView();
  // Every change of grouping, filters or page size is also remembered for this folder.
  const onSearchChange: MailSearchChange = (patch, options) => {
    navigate(patch, options);
    if (VIEW_KEYS.some((key) => key in patch)) {
      saveView.mutate({ folder, view: searchToView({ ...search, ...patch }) });
    }
  };
  const isSearch = (search.q?.trim() ?? "") !== "";
  const page = search.page ?? 1;
  const size = search.size ?? DEFAULT_PAGE_SIZE;
  const listParams: MessageListParams = {
    folder: isSearch ? search.folder : folder,
    project: search.project,
    q: search.q,
    since: search.period === undefined ? undefined : periodStart(search.period),
    limit: size,
    offset: (page - 1) * size,
  };
  const selectedId = search.message ?? null;
  const markRead = useMarkMessageRead();
  const { data: list } = useQuery(messageQueries.list(listParams));
  const isFiltered = isSearch || search.project !== undefined || search.period !== undefined;
  const isEmptyFolder = !isFiltered && list !== undefined && list.total === 0;
  const title = FOLDER_LABELS[folder];

  const [storedSelection, setSelection] = useState<Selection>(EMPTY_SELECTION);
  // Messages that left the list (moved, deleted, filtered) drop out of the selection.
  const selection = keepVisible(
    storedSelection,
    (list?.messages ?? []).map((message) => message.id),
  );
  const selectedMessages =
    selection.ids.size > 1
      ? (list?.messages ?? []).filter((message) => selection.ids.has(message.id))
      : [];

  const openMessage = (id: number, isRead: boolean) => {
    onSearchChange({ message: id });
    if (!isRead) {
      markRead.mutate(id);
    }
  };
  const handleSelect = (message: MessageSummary) => {
    setSelection(selectOne(message.id));
    openMessage(message.id, message.isRead);
  };
  // One message left selected is opened; several show what can be done with them.
  const handleSelectionChange = (next: Selection) => {
    setSelection(next);
    const [only] = next.ids;
    const message = list?.messages.find((candidate) => candidate.id === only);
    if (next.ids.size === 1 && message !== undefined) {
      openMessage(message.id, message.isRead);
    } else if (selectedId !== null) {
      onSearchChange({ message: undefined });
    }
  };
  const clearSelection = () => {
    setSelection(EMPTY_SELECTION);
    onSearchChange({ message: undefined });
  };

  return (
    <div className="grid h-full min-h-0 grid-rows-[auto_1fr]">
      <title>{`${title} – ${APP_NAME}`}</title>
      <PageHeader title={title}>
        {isEmptyFolder ? null : <MailStatus params={listParams} isSearch={isSearch} />}
        {isSearch ? (
          <MailFolderFilter
            folder={search.folder}
            onChange={(choice) =>
              onSearchChange({ folder: choice, message: undefined, page: undefined })
            }
          />
        ) : null}
        <MailSearchBox initialQuery={search.q ?? ""} onSearchChange={onSearchChange} />
      </PageHeader>
      {isEmptyFolder ? (
        <div className="flex min-h-0 overflow-y-auto">
          <MailEmptyFolder folder={folder} />
        </div>
      ) : (
        <div className="grid min-h-0 grid-cols-[minmax(16rem,24rem)_1fr]">
          <section
            aria-label="Message list"
            className="grid min-h-0 grid-rows-[auto_1fr_auto] border-r"
          >
            <MailListHeader
              search={search}
              onSearchChange={onSearchChange}
              showEmptyTrash={folder === "trash" && !isSearch}
              onTrashEmptied={clearSelection}
            />
            <div className="min-h-0 overflow-y-auto">
              <MailMessageList
                params={listParams}
                view={search.view ?? "all"}
                showFolder={isSearch}
                selectedId={selectedId}
                selection={selection}
                onSelectionChange={handleSelectionChange}
                onSelect={handleSelect}
                onRemoved={(id) => {
                  if (id === selectedId) {
                    clearSelection();
                  }
                }}
              />
            </div>
            {list === undefined || list.total === 0 ? null : (
              <MailPagination
                total={list.total}
                page={page}
                size={size}
                onChange={(nextPage, nextSize) =>
                  onSearchChange({
                    page: nextPage === 1 ? undefined : nextPage,
                    size: nextSize === DEFAULT_PAGE_SIZE ? undefined : nextSize,
                    message: undefined,
                  })
                }
              />
            )}
          </section>
          <section
            aria-label="Selected message"
            className="flex min-h-0 flex-col overflow-hidden p-4"
          >
            {selectedMessages.length > 1 ? (
              <MailSelectionPanel messages={selectedMessages} onClear={clearSelection} />
            ) : selectedId === null ? (
              <p className="m-auto text-muted-foreground">Select a message to read it.</p>
            ) : (
              <MailMessageView key={selectedId} messageId={selectedId} onClose={clearSelection} />
            )}
          </section>
        </div>
      )}
    </div>
  );
}
