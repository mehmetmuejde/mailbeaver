import { Fragment, type ReactNode } from "react";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenu as ContextMenuRoot,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/shared/ui/primitives/context-menu";

type ContextMenuAction = {
  label: string;
  icon: ReactNode;
  onSelect: () => void;
  destructive?: boolean;
  disabled?: boolean;
};

/** Actions in groups; a separator is drawn between groups. Empty groups are skipped. */
export type ContextMenuGroups = readonly (readonly ContextMenuAction[])[];

type ContextMenuProps = {
  /** The element the menu belongs to; it must be focusable for keyboard users. */
  children: ReactNode;
  groups: ContextMenuGroups;
};

/**
 * Right-click menu. Keyboard users open it with the context-menu key or Shift+F10 on
 * the focused element; arrow keys move, Enter selects, Escape closes.
 */
export function ContextMenu({ children, groups }: ContextMenuProps) {
  const visible = groups.filter((group) => group.length > 0);
  return (
    <ContextMenuRoot>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        {visible.map((group, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: groups are static per render and never reorder.
          <Fragment key={index}>
            {index > 0 ? <ContextMenuSeparator /> : null}
            {group.map((action) => (
              <ContextMenuItem
                key={action.label}
                variant={action.destructive ? "destructive" : "default"}
                disabled={action.disabled}
                onSelect={action.onSelect}
              >
                <span aria-hidden="true" className="inline-flex">
                  {action.icon}
                </span>
                {action.label}
              </ContextMenuItem>
            ))}
          </Fragment>
        ))}
      </ContextMenuContent>
    </ContextMenuRoot>
  );
}
