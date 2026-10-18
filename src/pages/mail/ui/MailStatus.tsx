import { useQuery } from "@tanstack/react-query";
import { type MessageListParams, messageQueries } from "@/shared/api";
import { CountBadge } from "@/shared/ui/badge";

type MailStatusProps = {
  params: MessageListParams;
  /** Counts search results instead of messages. */
  isSearch?: boolean;
};

/** Counts of the current list as labels next to the title; nothing when empty. */
export function MailStatus({ params, isSearch = false }: MailStatusProps) {
  const { data } = useQuery(messageQueries.list(params));
  return (
    <p role="status" className="flex flex-1 items-center gap-2">
      {data === undefined ? null : (
        <>
          <CountBadge count={data.unread} label="unread" tone="unread" showLabel />
          <CountBadge
            count={data.total}
            label={
              isSearch
                ? data.total === 1
                  ? "result"
                  : "results"
                : data.total === 1
                  ? "message"
                  : "messages"
            }
            showLabel
          />
        </>
      )}
    </p>
  );
}
