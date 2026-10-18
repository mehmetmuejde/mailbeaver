import { Badge } from "@/shared/ui/primitives/badge";

type StatusBadgeProps = {
  tone: "success" | "danger";
  children: string;
};

/** A short outcome such as an HTTP status; the text carries the meaning, color helps. */
export function StatusBadge({ tone, children }: StatusBadgeProps) {
  return (
    <Badge
      variant="outline"
      className={
        tone === "success"
          ? "border-success/40 bg-success/10 text-success"
          : "border-destructive/40 bg-destructive/10 text-destructive"
      }
    >
      {children}
    </Badge>
  );
}
