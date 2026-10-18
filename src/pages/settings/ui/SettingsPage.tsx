import { APP_NAME } from "@/shared/config";
import { PageHeader } from "@/shared/ui/page-header";
import { type TabItem, Tabs } from "@/shared/ui/tabs";
import { SETTINGS_TABS, type SettingsTab } from "../model/settings-search";
import { SettingsDangerZone } from "./SettingsDangerZone";
import { SettingsDeveloper } from "./SettingsDeveloper";
import { SettingsGeneral } from "./SettingsGeneral";
import { SettingsLogs } from "./SettingsLogs";
import { SettingsNotifications } from "./SettingsNotifications";
import { SettingsProjects } from "./SettingsProjects";
import { SettingsStats } from "./SettingsStats";
import { SettingsTrash } from "./SettingsTrash";
import { SettingsTray } from "./SettingsTray";

const TABS: readonly TabItem[] = [
  { id: "general", label: "General", panel: <SettingsGeneral /> },
  { id: "projects", label: "Projects", panel: <SettingsProjects /> },
  { id: "trash", label: "Trash", panel: <SettingsTrash /> },
  { id: "tray", label: "Tray menu", panel: <SettingsTray /> },
  { id: "notifications", label: "Notifications", panel: <SettingsNotifications /> },
  { id: "stats", label: "Statistics", panel: <SettingsStats /> },
  { id: "api", label: "API & MCP", panel: <SettingsDeveloper /> },
  { id: "logs", label: "Logs", panel: <SettingsLogs /> },
  { id: "danger", label: "Danger zone", panel: <SettingsDangerZone /> },
];

function isSettingsTab(id: string): id is SettingsTab {
  return SETTINGS_TABS.some((tab) => tab === id);
}

type SettingsPageProps = {
  tab: SettingsTab;
  onTabChange: (tab: SettingsTab) => void;
};

/** Each section is a tab; the selected one lives in the URL. */
export function SettingsPage({ tab, onTabChange }: SettingsPageProps) {
  return (
    <div className="grid h-full min-h-0 grid-rows-[auto_1fr]">
      <title>{`Settings – ${APP_NAME}`}</title>
      <PageHeader title="Settings" />
      <div className="min-h-0 overflow-y-auto px-8 py-6">
        <div className="mx-auto max-w-3xl">
          <Tabs
            label="Settings sections"
            tabs={TABS}
            value={tab}
            onValueChange={(id) => {
              if (isSettingsTab(id)) {
                onTabChange(id);
              }
            }}
          />
        </div>
      </div>
    </div>
  );
}
