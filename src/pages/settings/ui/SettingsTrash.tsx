import { useQuery } from "@tanstack/react-query";
import { settingsQueries } from "@/shared/api";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { useUpdateSettings } from "../api/use-update-settings";
import { RETENTION_CHOICES } from "../model/retention-options";

const OPTIONS: readonly SelectOption[] = RETENTION_CHOICES.map((choice) => ({
  value: choice,
  label: choice === "off" ? "Never" : `After ${choice} ${choice === "1" ? "day" : "days"}`,
}));

/** Saves on change; the server deletes expired messages hourly and right after a change. */
export function SettingsTrash() {
  const { data: settings, isError } = useQuery(settingsQueries.settings());
  const update = useUpdateSettings();

  const current =
    settings?.trashRetentionDays == null ? "off" : String(settings.trashRetentionDays);
  const handleChange = (value: string) => {
    if (settings !== undefined) {
      update.mutate({ ...settings, trashRetentionDays: value === "off" ? null : Number(value) });
    }
  };

  return (
    <section aria-labelledby="trash-heading" className="grid gap-3">
      <h2 id="trash-heading" className="text-base font-semibold">
        Trash
      </h2>
      <SelectField
        label="Empty trash automatically"
        options={
          OPTIONS.some((option) => option.value === current)
            ? OPTIONS
            : [...OPTIONS, { value: current, label: `After ${current} days` }]
        }
        value={current}
        disabled={settings === undefined || update.isPending}
        onChange={handleChange}
      />
      <p role="status" className="text-muted-foreground">
        {update.isSuccess ? "Saved." : null}
      </p>
      {isError ? <p role="alert">Settings could not be loaded.</p> : null}
      {update.isError ? <p role="alert">The setting could not be saved.</p> : null}
    </section>
  );
}
