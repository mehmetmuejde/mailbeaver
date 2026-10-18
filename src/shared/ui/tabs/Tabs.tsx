import { type ReactNode, useState } from "react";
import {
  TabsContent,
  TabsList,
  Tabs as TabsPrimitive,
  TabsTrigger,
} from "@/shared/ui/primitives/tabs";

export type TabItem = {
  id: string;
  label: string;
  /** Rendered only while the tab is active. */
  panel: ReactNode;
  /** Controls for this tab, shown at the right of the tab row while it is active. */
  actions?: ReactNode;
};

type TabsProps = {
  /** Accessible name of the tab list. */
  label: string;
  tabs: readonly TabItem[];
  defaultTabId?: string;
  /** Controls the selection, e.g. from the URL; leave out to let the tabs keep it. */
  value?: string;
  onValueChange?: (id: string) => void;
  /** `buttons` draws the tabs like a joined button group (same look as SegmentedControl). */
  appearance?: "default" | "buttons";
};

// Matches the outline, small, joined look of SegmentedControl (toggle group).
const BUTTON_LIST_CLASS = "h-7 gap-0 rounded-lg bg-transparent p-0";
const BUTTON_TRIGGER_CLASS =
  "h-7 flex-none rounded-none border border-l-0 border-input px-2.5 text-[0.8rem] text-foreground shadow-none! first:rounded-l-lg first:border-l last:rounded-r-lg hover:bg-muted data-active:bg-muted dark:data-active:bg-muted";

/** WAI-ARIA tabs with automatic activation: arrow keys, Home and End move the selection. */
export function Tabs({
  label,
  tabs,
  defaultTabId,
  value,
  onValueChange,
  appearance = "default",
}: TabsProps) {
  const [ownValue, setOwnValue] = useState(defaultTabId ?? tabs[0]?.id);
  const requested = value ?? ownValue;
  const selected = tabs.some((tab) => tab.id === requested) ? requested : tabs[0]?.id;
  const actions = tabs.find((tab) => tab.id === selected)?.actions;
  const buttons = appearance === "buttons";

  const handleChange = (id: string) => {
    setOwnValue(id);
    onValueChange?.(id);
  };

  return (
    <TabsPrimitive value={selected} onValueChange={handleChange} className="min-h-0 flex-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <TabsList aria-label={label} className={buttons ? BUTTON_LIST_CLASS : undefined}>
          {tabs.map((tab) => (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
              className={buttons ? BUTTON_TRIGGER_CLASS : undefined}
            >
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {actions === undefined ? null : actions}
      </div>
      {tabs.map((tab) => (
        <TabsContent
          key={tab.id}
          value={tab.id}
          className="flex min-h-0 flex-col overflow-auto pt-3"
        >
          {tab.panel}
        </TabsContent>
      ))}
    </TabsPrimitive>
  );
}
