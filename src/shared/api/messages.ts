import { keepPreviousData, queryOptions } from "@tanstack/react-query";
import { z } from "zod";
import { apiUrl, request, requestJson, sendJson } from "./client";

export const FOLDERS = ["inbox", "archive", "trash"] as const;
const folderSchema = z.enum(FOLDERS);
export type Folder = z.infer<typeof folderSchema>;

const messageSummarySchema = z.object({
  id: z.number().int(),
  /** Unix timestamp in milliseconds. */
  receivedAt: z.number(),
  from: z.string(),
  to: z.array(z.string()),
  subject: z.string(),
  size: z.number(),
  hasAttachments: z.boolean(),
  isRead: z.boolean(),
  folder: folderSchema,
  /** Assigned by the project rules; `null` when no rule matches. */
  project: z.object({ id: z.number().int(), name: z.string() }).nullable(),
  /** The start of the plain-text body. */
  preview: z.string(),
});
export type MessageSummary = z.infer<typeof messageSummarySchema>;

const messagePageSchema = z.object({
  total: z.number(),
  unread: z.number(),
  messages: z.array(messageSummarySchema),
});
export type MessagePage = z.infer<typeof messagePageSchema>;

const headerLineSchema = z.object({ name: z.string(), value: z.string() });
export type HeaderLine = z.infer<typeof headerLineSchema>;

const attachmentInfoSchema = z.object({
  index: z.number().int(),
  filename: z.string(),
  contentType: z.string(),
  size: z.number(),
});
export type AttachmentInfo = z.infer<typeof attachmentInfoSchema>;

const messageDetailSchema = messageSummarySchema.extend({
  envelopeFrom: z.string(),
  envelopeTo: z.array(z.string()),
  /** SMTP AUTH username, if the client authenticated. */
  authUser: z.string().nullable(),
  headers: z.array(headerLineSchema),
  text: z.string().nullable(),
  html: z.string().nullable(),
  attachments: z.array(attachmentInfoSchema),
});
export type MessageDetail = z.infer<typeof messageDetailSchema>;

export const countsSchema = z.object({ total: z.number(), unread: z.number() });

const folderCountsSchema = z.object({
  inbox: countsSchema,
  archive: countsSchema,
  trash: countsSchema,
});
type FolderCounts = z.infer<typeof folderCountsSchema>;

/** Selects messages without a project in a list. */
export const UNASSIGNED_PROJECT = "none";

export type MessageListParams = {
  /** `undefined` searches every folder. */
  folder?: Folder;
  /** `undefined` lists every project, `"none"` the unassigned messages. */
  project?: number | typeof UNASSIGNED_PROJECT;
  q?: string;
  /** Unix milliseconds: only messages received at or after this time. */
  since?: number;
  limit: number;
  offset?: number;
};

export function fetchMessages(params: MessageListParams): Promise<MessagePage> {
  const search = new URLSearchParams({ limit: String(params.limit) });
  if (params.folder !== undefined) {
    search.set("folder", params.folder);
  }
  if (params.project !== undefined) {
    search.set("project", String(params.project));
  }
  if (params.q !== undefined && params.q.trim() !== "") {
    search.set("q", params.q.trim());
  }
  if (params.since !== undefined) {
    search.set("since", String(params.since));
  }
  if (params.offset !== undefined && params.offset > 0) {
    search.set("offset", String(params.offset));
  }
  return requestJson(`/api/messages?${search}`, messagePageSchema);
}

function fetchMessage(id: number): Promise<MessageDetail> {
  return requestJson(`/api/messages/${id}`, messageDetailSchema);
}

async function fetchRawMessage(id: number): Promise<string> {
  const response = await request(`/api/messages/${id}/raw`);
  return response.text();
}

function fetchFolderCounts(): Promise<FolderCounts> {
  return requestJson("/api/folders", folderCountsSchema);
}

export async function markMessageRead(id: number): Promise<void> {
  await request(`/api/messages/${id}/read`, { method: "POST" });
}

export async function markMessageUnread(id: number): Promise<void> {
  await request(`/api/messages/${id}/unread`, { method: "POST" });
}

export async function moveMessage({ id, folder }: { id: number; folder: Folder }): Promise<void> {
  await sendJson(`/api/messages/${id}/move`, "POST", { folder });
}

/** Deletes a message for good; the server only allows it for messages in Trash. */
export async function deleteMessagePermanently(id: number): Promise<void> {
  await request(`/api/messages/${id}`, { method: "DELETE" });
}

/** Marks every unread message of a folder read; without a folder, of all folders. */
export async function markAllRead(folder?: Folder): Promise<void> {
  await sendJson("/api/messages/mark-read", "POST", { folder: folder ?? null });
}

export async function emptyTrash(): Promise<void> {
  await request("/api/trash", { method: "DELETE" });
}

export async function deleteAllMessages(): Promise<void> {
  await request("/api/messages", { method: "DELETE" });
}

/** Removes every message, project and setting: Mailbeaver starts over. */
export async function resetAllData(): Promise<void> {
  await request("/api/data", { method: "DELETE" });
}

/** The message as an `.eml` file named after its subject. */
export function emlDownloadUrl(id: number): string {
  return apiUrl(`/api/messages/${id}/raw?download=true`);
}

export function attachmentUrl(id: number, index: number): string {
  return apiUrl(`/api/messages/${id}/attachments/${index}`);
}

export function mailboxEventsUrl(): string {
  return apiUrl("/api/events");
}

export const messageKeys = {
  all: ["messages"] as const,
  lists: () => [...messageKeys.all, "list"] as const,
  list: (params: MessageListParams) => [...messageKeys.lists(), params] as const,
  detail: (id: number) => [...messageKeys.all, "detail", id] as const,
  raw: (id: number) => [...messageKeys.all, "raw", id] as const,
  folders: () => [...messageKeys.all, "folders"] as const,
};

export const messageQueries = {
  // A new search or filter keeps showing the previous list until its result arrives.
  list: (params: MessageListParams) =>
    queryOptions({
      queryKey: messageKeys.list(params),
      queryFn: () => fetchMessages(params),
      placeholderData: keepPreviousData,
    }),
  detail: (id: number) =>
    queryOptions({ queryKey: messageKeys.detail(id), queryFn: () => fetchMessage(id) }),
  // The raw source of a stored message never changes.
  raw: (id: number) =>
    queryOptions({
      queryKey: messageKeys.raw(id),
      queryFn: () => fetchRawMessage(id),
      staleTime: Number.POSITIVE_INFINITY,
    }),
  folders: () => queryOptions({ queryKey: messageKeys.folders(), queryFn: fetchFolderCounts }),
};
