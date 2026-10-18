import { useForm } from "@tanstack/react-form";
import { RotateCw } from "lucide-react";
import { useState } from "react";
import { fieldError } from "@/shared/lib/form";
import { restartApp, saveServerPorts } from "@/shared/lib/platform";
import { Button } from "@/shared/ui/button";
import { TextField } from "@/shared/ui/text-field";
import { portFormSchema } from "../model/port-form";

/**
 * The HTTP API is down after a port conflict, so the desktop app saves the new ports
 * directly and restarts (ADR-0031).
 */
export function ServerErrorPortForm() {
  const [failure, setFailure] = useState<string | null>(null);
  const form = useForm({
    defaultValues: { smtpPort: "1026", httpPort: "8026" },
    validators: { onSubmit: portFormSchema },
    onSubmit: async ({ value }) => {
      const ports = portFormSchema.parse(value);
      setFailure(null);
      try {
        await saveServerPorts(ports.smtpPort, ports.httpPort);
        await restartApp();
      } catch (error) {
        setFailure(error instanceof Error ? error.message : String(error));
      }
    },
  });

  return (
    <form
      noValidate
      aria-label="Choose other ports"
      className="grid gap-3 rounded-lg border p-3"
      onSubmit={(event) => {
        event.preventDefault();
        form.handleSubmit().catch(() => undefined);
      }}
    >
      <div className="grid gap-3 sm:grid-cols-2">
        <form.Field name="smtpPort">
          {(field) => (
            <TextField
              label="SMTP port"
              inputMode="numeric"
              value={field.state.value}
              onChange={field.handleChange}
              onBlur={field.handleBlur}
              error={fieldError(field.state.meta.errors)}
            />
          )}
        </form.Field>
        <form.Field name="httpPort">
          {(field) => (
            <TextField
              label="HTTP port"
              inputMode="numeric"
              value={field.state.value}
              onChange={field.handleChange}
              onBlur={field.handleBlur}
              error={fieldError(field.state.meta.errors)}
            />
          )}
        </form.Field>
      </div>
      {failure === null ? null : <p role="alert">{failure}</p>}
      <div>
        <form.Subscribe selector={(state) => state.isSubmitting}>
          {(isSubmitting) => (
            <Button type="submit" variant="default" icon={<RotateCw />} disabled={isSubmitting}>
              Save and restart
            </Button>
          )}
        </form.Subscribe>
      </div>
    </form>
  );
}
