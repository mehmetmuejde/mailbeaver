import { z } from "zod";
import { FOLDERS, PAGE_SIZES, PERIODS, UNASSIGNED_PROJECT } from "@/shared/api";

export const SEARCH_MAX_LENGTH = 256;

const MAIL_VIEWS = ["all", "sender", "project"] as const;

export { PAGE_SIZES, PERIODS };
export type PageSize = (typeof PAGE_SIZES)[number];
export const DEFAULT_PAGE_SIZE: PageSize = 50;

export type Period = (typeof PERIODS)[number];
export type MailView = (typeof MAIL_VIEWS)[number];

/** URL state of a folder view; invalid values fall back instead of breaking the page. */
export const mailSearchSchema = z.object({
  q: z.string().max(SEARCH_MAX_LENGTH).optional().catch(undefined),
  view: z.enum(MAIL_VIEWS).optional().catch(undefined),
  /** Absent: all projects; `"none"`: messages without a project; otherwise a project id. */
  project: z
    .union([z.literal(UNASSIGNED_PROJECT), z.number().int().positive()])
    .optional()
    .catch(undefined),
  message: z.number().int().positive().optional().catch(undefined),
  /** 1-based page of the list. */
  page: z.number().int().min(1).optional().catch(undefined),
  size: z
    .union(PAGE_SIZES.map((size) => z.literal(size)))
    .optional()
    .catch(undefined),
  /** Only messages received in this period; absent shows all. */
  period: z.enum(PERIODS).optional().catch(undefined),
  /** While searching: narrows the results to one folder; absent searches all. */
  folder: z.enum(FOLDERS).optional().catch(undefined),
});
export type MailSearch = z.infer<typeof mailSearchSchema>;

/** Merges a change into the URL state; `replace` avoids a history entry per keystroke. */
export type MailSearchChange = (
  patch: Partial<MailSearch>,
  options?: { replace?: boolean },
) => void;
