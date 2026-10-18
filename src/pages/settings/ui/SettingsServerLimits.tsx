import { useForm } from "@tanstack/react-form";
import { useMutation, useQuery } from "@tanstack/react-query";
import { RotateCw, Save } from "lucide-react";
import { ApiError, type ServerSettings, settingsQueries } from "@/shared/api";
import { fieldError } from "@/shared/lib/form";
import { restartApp } from "@/shared/lib/platform";
import { Button } from "@/shared/ui/button";
import { TextField } from "@/shared/ui/text-field";
import { useSaveServerSettings } from "../api/use-save-server-settings";
import {
  ENVIRONMENT_VARIABLES,
  SERVER_DEFAULTS,
  type ServerSettingKey,
  serverSettingsFormSchema,
  toFormValues,
} from "../model/server-settings-form";

const FIELDS: readonly { key: ServerSettingKey; label: string; unit: string }[] = [
  { key: "smtpPort", label: "SMTP port", unit: "" },
  { key: "httpPort", label: "HTTP port", unit: "" },
  { key: "maxMessages", label: "Messages kept", unit: " messages" },
  { key: "maxMessageMb", label: "Largest message", unit: " MB" },
];

/** Loads the saved values first, so the form starts from them. */
export function SettingsServerLimits() {
  const { data: saved } = useQuery(settingsQueries.server());
  const { data: info } = useQuery(settingsQueries.info());
  if (saved === undefined || info === undefined) {
    return null;
  }
  return (
    <ServerLimitsForm
      saved={saved}
      fromEnvironment={info.fromEnvironment}
      canRestart={info.host === "desktop"}
    />
  );
}

type ServerLimitsFormProps = {
  saved: ServerSettings;
  fromEnvironment: readonly ServerSettingKey[];
  canRestart: boolean;
};

/** Ports and limits for the next start; an environment variable wins (ADR-0031). */
function ServerLimitsForm({ saved, fromEnvironment, canRestart }: ServerLimitsFormProps) {
  const save = useSaveServerSettings();
  const restart = useMutation({ mutationFn: restartApp });
  const form = useForm({
    defaultValues: toFormValues(saved),
    validators: { onSubmit: serverSettingsFormSchema },
    onSubmit: async ({ value }) => {
      await save.mutateAsync(serverSettingsFormSchema.parse(value));
    },
  });
  const serverError =
    save.error instanceof ApiError && save.error.detail !== null
      ? save.error.detail
      : "The settings could not be saved.";

  return (
    <section aria-labelledby="limits-heading" className="grid gap-3">
      <h2 id="limits-heading" className="text-base font-semibold">
        Ports and limits
      </h2>
      <p className="text-muted-foreground">
        Apply at the next start. Leave a field empty for the default; an environment variable
        overrides the value here.
      </p>
      <form
        noValidate
        aria-label="Ports and limits"
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          form.handleSubmit().catch(() => undefined);
        }}
      >
        <div className="grid gap-3 sm:grid-cols-2">
          {FIELDS.map(({ key, label, unit }) => {
            const locked = fromEnvironment.includes(key);
            return (
              <form.Field key={key} name={key}>
                {(field) => (
                  <TextField
                    label={label}
                    inputMode="numeric"
                    placeholder={`${SERVER_DEFAULTS[key]}${unit}`}
                    value={field.state.value}
                    onChange={field.handleChange}
                    onBlur={field.handleBlur}
                    disabled={locked}
                    description={locked ? `Set by ${ENVIRONMENT_VARIABLES[key]}` : undefined}
                    error={fieldError(field.state.meta.errors)}
                  />
                )}
              </form.Field>
            );
          })}
        </div>
        {save.isError ? <p role="alert">{serverError}</p> : null}
        <div className="flex flex-wrap items-center gap-2">
          <form.Subscribe selector={(state) => state.isSubmitting}>
            {(isSubmitting) => (
              <Button type="submit" variant="default" icon={<Save />} disabled={isSubmitting}>
                {isSubmitting ? "Saving…" : "Save"}
              </Button>
            )}
          </form.Subscribe>
          {save.isSuccess && canRestart ? (
            <Button
              icon={<RotateCw />}
              disabled={restart.isPending}
              onClick={() => restart.mutate()}
            >
              Restart now
            </Button>
          ) : null}
          <p role="status" className="text-sm text-muted-foreground">
            {save.isSuccess
              ? canRestart
                ? "Saved. Restart Mailbeaver to apply."
                : "Saved. Restart the server (or its container) to apply."
              : null}
          </p>
        </div>
        {restart.isError ? <p role="alert">Mailbeaver could not restart.</p> : null}
      </form>
    </section>
  );
}
