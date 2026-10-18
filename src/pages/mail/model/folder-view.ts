import { type FolderView, UNASSIGNED_PROJECT } from "@/shared/api";
import { DEFAULT_PAGE_SIZE, type MailSearch } from "./mail-search";

/** The URL parameters that make up a folder's saved view. */
export const VIEW_KEYS = ["view", "project", "period", "size"] as const;

export function hasViewParams(search: MailSearch): boolean {
  return VIEW_KEYS.some((key) => search[key] !== undefined);
}

/** URL parameters for a saved view; defaults are left out so the URL stays short. */
export function viewToSearch(view: FolderView): Partial<MailSearch> {
  const project =
    view.project === null
      ? undefined
      : view.project === UNASSIGNED_PROJECT
        ? UNASSIGNED_PROJECT
        : Number(view.project);
  return {
    view: view.grouping === "all" ? undefined : view.grouping,
    project,
    period: view.period ?? undefined,
    size: view.pageSize === DEFAULT_PAGE_SIZE ? undefined : view.pageSize,
  };
}

export function searchToView(search: MailSearch): FolderView {
  return {
    grouping: search.view ?? "all",
    project: search.project === undefined ? null : String(search.project),
    period: search.period ?? null,
    pageSize: search.size ?? DEFAULT_PAGE_SIZE,
  };
}
