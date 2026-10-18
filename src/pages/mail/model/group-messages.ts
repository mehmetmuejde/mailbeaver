import type { MessageSummary } from "@/shared/api";
import type { MailView } from "./mail-search";

export const UNKNOWN_SENDER = "(unknown sender)";
export const NO_PROJECT = "No project";

export type MessageGroup = {
  key: string;
  /** `null` for the ungrouped view. */
  label: string | null;
  messages: MessageSummary[];
};

function groupKey(message: MessageSummary, view: Exclude<MailView, "all">): string {
  if (view === "sender") {
    return message.from || UNKNOWN_SENDER;
  }
  return message.project?.name ?? NO_PROJECT;
}

/** Groups keep the order of their newest message, so the newest group comes first. */
export function groupMessages(messages: readonly MessageSummary[], view: MailView): MessageGroup[] {
  if (view === "all") {
    return [{ key: "all", label: null, messages: [...messages] }];
  }
  const groups = new Map<string, MessageSummary[]>();
  for (const message of messages) {
    const key = groupKey(message, view);
    groups.set(key, [...(groups.get(key) ?? []), message]);
  }
  return [...groups].map(([key, grouped]) => ({ key, label: key, messages: grouped }));
}
