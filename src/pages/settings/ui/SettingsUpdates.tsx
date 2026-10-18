import { useQuery } from "@tanstack/react-query";
import { settingsQueries } from "@/shared/api";
import { SwitchField } from "@/shared/ui/switch-field";
import { useUpdateSettings } from "../api/use-update-settings";

/** Opt-in, because asking GitHub tells it the IP address (ADR-0032). */
export function SettingsUpdates() {
  const { data: settings } = useQuery(settingsQueries.settings());
  const { data: update } = useQuery(settingsQueries.update());
  const save = useUpdateSettings();

  return (
    <section aria-labelledby="updates-heading" className="grid gap-3">
      <h2 id="updates-heading" className="text-base font-semibold">
        Updates
      </h2>
      <SwitchField
        label="Check for updates"
        description="Asks GitHub once a day for a newer release; GitHub sees your IP address. Nothing is installed automatically."
        checked={settings?.updateCheck ?? false}
        disabled={settings === undefined || save.isPending}
        onCheckedChange={(updateCheck) => {
          if (settings !== undefined) {
            save.mutate({ ...settings, updateCheck });
          }
        }}
      />
      {settings?.updateCheck && update?.latestVersion != null ? (
        <p className="text-sm text-muted-foreground">
          {update.updateAvailable
            ? `Version ${update.latestVersion} is available. With Homebrew: brew upgrade --cask mailbeaver`
            : `Mailbeaver ${update.currentVersion} is up to date.`}
        </p>
      ) : null}
      {save.isError ? <p role="alert">The setting could not be saved.</p> : null}
    </section>
  );
}
