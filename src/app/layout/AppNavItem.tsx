import { Link } from "@tanstack/react-router";
import { CheckCheck } from "lucide-react";
import type { ReactNode } from "react";
import { CountBadge } from "@/shared/ui/badge";
import { ContextMenu } from "@/shared/ui/context-menu";

const LINK_CLASS =
  "flex items-center gap-2 rounded-md px-3 py-2 hover:bg-muted aria-[current=page]:bg-accent aria-[current=page]:font-semibold";

type AppNavItemProps = {
  to: "/inbox" | "/archive" | "/trash" | "/settings";
  icon: ReactNode;
  label: string;
  counts?: { total: number; unread: number };
  /** Folders offer "Mark all as read" on right-click. */
  onMarkAllRead?: () => void;
};

export function AppNavItem({ to, icon, label, counts, onMarkAllRead }: AppNavItemProps) {
  const link = (
    <Link to={to} className={LINK_CLASS}>
      <span aria-hidden="true" className="[&_svg]:size-4">
        {icon}
      </span>
      {label}
      {/* The leading space keeps the accessible name readable: "Inbox 3 unread 12 messages". */}
      {counts === undefined ? null : (
        <>
          {" "}
          <span className="ml-auto flex gap-1">
            <CountBadge count={counts.unread} label="unread" tone="unread" />{" "}
            <CountBadge count={counts.total} label={counts.total === 1 ? "message" : "messages"} />
          </span>
        </>
      )}
    </Link>
  );

  return (
    <li>
      {onMarkAllRead === undefined ? (
        link
      ) : (
        <ContextMenu
          groups={[
            [
              {
                label: "Mark all as read",
                icon: <CheckCheck />,
                disabled: (counts?.unread ?? 0) === 0,
                onSelect: onMarkAllRead,
              },
            ],
          ]}
        >
          {link}
        </ContextMenu>
      )}
    </li>
  );
}
