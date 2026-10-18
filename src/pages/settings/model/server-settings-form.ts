import { z } from "zod";
import type { ServerSettings } from "@/shared/api";

/** What the server uses when nothing is saved and no environment variable is set. */
export const SERVER_DEFAULTS = {
  smtpPort: 1026,
  httpPort: 8026,
  maxMessages: 5000,
  maxMessageMb: 25,
} as const;

export const ENVIRONMENT_VARIABLES = {
  smtpPort: "MAILBEAVER_SMTP_PORT",
  httpPort: "MAILBEAVER_HTTP_PORT",
  maxMessages: "MAILBEAVER_MAX_MESSAGES",
  maxMessageMb: "MAILBEAVER_MAX_MESSAGE_MB",
} as const;

export type ServerSettingKey = keyof typeof SERVER_DEFAULTS;

/** Empty keeps the default; otherwise a whole number in range. */
function optionalNumber(max: number, message: string) {
  return z
    .string()
    .trim()
    .refine(
      (value) =>
        value === "" || (/^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= max),
      {
        error: message,
      },
    )
    .transform((value) => (value === "" ? null : Number(value)));
}

const PORT_ERROR = "Enter a port between 1 and 65535, or leave empty";

export const serverSettingsFormSchema = z
  .object({
    smtpPort: optionalNumber(65_535, PORT_ERROR),
    httpPort: optionalNumber(65_535, PORT_ERROR),
    maxMessages: optionalNumber(1_000_000, "Enter 1 to 1,000,000, or leave empty"),
    maxMessageMb: optionalNumber(1024, "Enter 1 to 1024 MB, or leave empty"),
  })
  .refine(
    (values) =>
      (values.smtpPort ?? SERVER_DEFAULTS.smtpPort) !==
      (values.httpPort ?? SERVER_DEFAULTS.httpPort),
    { error: "SMTP and HTTP need different ports", path: ["smtpPort"] },
  );

export type ServerSettingsFormValues = z.input<typeof serverSettingsFormSchema>;

export function toFormValues(settings: ServerSettings): ServerSettingsFormValues {
  const text = (value: number | null) => (value === null ? "" : String(value));
  return {
    smtpPort: text(settings.smtpPort),
    httpPort: text(settings.httpPort),
    maxMessages: text(settings.maxMessages),
    maxMessageMb: text(settings.maxMessageMb),
  };
}
