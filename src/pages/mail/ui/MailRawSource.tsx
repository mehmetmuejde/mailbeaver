import { useQuery } from "@tanstack/react-query";
import { messageQueries } from "@/shared/api";

type MailRawSourceProps = {
  messageId: number;
};

/** Loads the raw source only when its tab is opened. */
export function MailRawSource({ messageId }: MailRawSourceProps) {
  const { data, isPending, isError } = useQuery(messageQueries.raw(messageId));

  if (isPending) {
    return <p role="status">Loading source…</p>;
  }
  if (isError) {
    return <p role="alert">The source could not be loaded.</p>;
  }
  return <pre className="font-mono text-xs break-all whitespace-pre-wrap">{data}</pre>;
}
