import { Badge } from "@/shared/ui/primitives/badge";

type CountBadgeProps = {
  count: number;
  /** What is counted, e.g. "unread" or "messages"; spoken after the number. */
  label: string;
  /** `unread` stands out in the accent color. */
  tone?: "muted" | "unread";
  /** Shows the label next to the number instead of only to screen readers. */
  showLabel?: boolean;
};

/** A small count; hidden when there is nothing to count. */
export function CountBadge({ count, label, tone = "muted", showLabel = false }: CountBadgeProps) {
  if (count <= 0) {
    return null;
  }
  return (
    <Badge
      variant="secondary"
      className={`tabular-nums ${tone === "unread" ? "bg-unread text-primary-foreground" : ""}`}
    >
      {count} <span className={showLabel ? undefined : "sr-only"}>{label}</span>
    </Badge>
  );
}
