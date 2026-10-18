import { useQuery } from "@tanstack/react-query";
import { settingsQueries, TRAY_GROUPINGS } from "@/shared/api";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { useUpdateSettings } from "../api/use-update-settings";

const GROUPING_LABELS = {
  all: "Newest first",
  sender: "By sender",
  project: "By project",
} as const;

const OPTIONS: readonly SelectOption[] = TRAY_GROUPINGS.map((grouping) => ({
  value: grouping,
  label: GROUPING_LABELS[grouping],
}));

function isGrouping(value: string): value is (typeof TRAY_GROUPINGS)[number] {
  return TRAY_GROUPINGS.some((grouping) => grouping === value);
}

/** The desktop app lists the newest inbox messages in its tray menu. */
export function SettingsTray() {
  const { data: settings } = useQuery(settingsQueries.settings());
  const update = useUpdateSettings();

  const handleChange = (value: string) => {
    if (settings !== undefined && isGrouping(value)) {
      update.mutate({ ...settings, trayGrouping: value });
    }
  };

  return (
    <section aria-labelledby="tray-heading" className="grid gap-3">
      <h2 id="tray-heading" className="text-base font-semibold">
        Tray menu
      </h2>
      <p className="text-muted-foreground">
        The menu bar icon of the desktop app lists the ten newest inbox messages.
      </p>
      <SelectField
        label="Group messages"
        options={OPTIONS}
        value={settings?.trayGrouping ?? "all"}
        disabled={settings === undefined || update.isPending}
        onChange={handleChange}
      />
      {update.isError ? <p role="alert">The setting could not be saved.</p> : null}
    </section>
  );
}
