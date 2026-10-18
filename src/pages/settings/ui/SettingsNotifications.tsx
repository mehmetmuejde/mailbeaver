import { useMutation, useQuery } from "@tanstack/react-query";
import { BellRing } from "lucide-react";
import { NOTIFICATION_MODES, type Settings, settingsQueries } from "@/shared/api";
import { isDesktopApp, sendTestNotification } from "@/shared/lib/platform";
import { Button } from "@/shared/ui/button";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { SwitchField } from "@/shared/ui/switch-field";
import { useUpdateSettings } from "../api/use-update-settings";

type NotificationMode = Settings["notifications"];

const MODE_LABELS: Record<NotificationMode, string> = {
  off: "Off",
  all: "All new mail",
  projects: "Selected projects",
};

const OPTIONS: readonly SelectOption[] = NOTIFICATION_MODES.map((mode) => ({
  value: mode,
  label: MODE_LABELS[mode],
}));

function isMode(value: string): value is NotificationMode {
  return NOTIFICATION_MODES.some((mode) => mode === value);
}

/**
 * The desktop app notifies about new mail while its window is not in front and shows
 * the unread count on its icon. Saves on change.
 */
export function SettingsNotifications() {
  const { data: settings } = useQuery(settingsQueries.settings());
  const update = useUpdateSettings();
  const test = useMutation({ mutationFn: sendTestNotification });
  const disabled = settings === undefined || update.isPending;

  const change = (patch: Partial<Settings>) => {
    if (settings !== undefined) {
      update.mutate({ ...settings, ...patch });
    }
  };

  return (
    <section aria-labelledby="notifications-heading" className="grid gap-4">
      <h2 id="notifications-heading" className="text-base font-semibold">
        Notifications
      </h2>
      <p className="text-muted-foreground">
        The desktop app announces new inbox mail while its window is not in front. Mail arriving
        within a second shares one notification; a click opens it.
      </p>
      <SelectField
        label="Notify about"
        spread
        options={OPTIONS}
        value={settings?.notifications ?? "all"}
        disabled={disabled}
        onChange={(value) => {
          if (isMode(value)) {
            change({ notifications: value });
          }
        }}
      />
      {settings?.notifications === "projects" ? (
        <p className="text-sm text-muted-foreground">
          Choose the projects under Projects: “Notify about new mail”.
        </p>
      ) : null}
      <SwitchField
        label="Play a sound"
        checked={settings?.notificationSound ?? false}
        disabled={disabled || settings?.notifications === "off"}
        onCheckedChange={(checked) => change({ notificationSound: checked })}
      />
      <SwitchField
        label="Unread count on the app icon"
        description="Shows the number of unread inbox messages on the Dock icon."
        checked={settings?.unreadBadge ?? true}
        disabled={disabled}
        onCheckedChange={(checked) => change({ unreadBadge: checked })}
      />
      {isDesktopApp() ? (
        <div>
          <Button
            icon={<BellRing />}
            disabled={test.isPending}
            onClick={() => test.mutate(settings?.notificationSound ?? false)}
          >
            Send test notification
          </Button>
        </div>
      ) : null}
      {test.isError ? <p role="alert">The test notification could not be sent.</p> : null}
      {update.isError ? <p role="alert">The setting could not be saved.</p> : null}
    </section>
  );
}
