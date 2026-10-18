import type { LinkStatus } from "@/shared/api";
import { StatusBadge } from "@/shared/ui/status-badge";

type MailLinkStatusProps = {
  /** `undefined` until the link was checked. */
  status: LinkStatus | undefined;
};

export function MailLinkStatus({ status }: MailLinkStatusProps) {
  if (status === undefined) {
    return null;
  }
  if (status.status === null) {
    return <StatusBadge tone="danger">{status.error ?? "failed"}</StatusBadge>;
  }
  return (
    <StatusBadge tone={status.ok ? "success" : "danger"}>
      {status.ok ? `${status.status} OK` : String(status.status)}
    </StatusBadge>
  );
}
