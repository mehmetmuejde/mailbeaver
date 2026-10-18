import { z } from "zod";

const port = z
  .string()
  .trim()
  .refine((value) => /^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= 65_535, {
    error: "Enter a port between 1 and 65535",
  })
  .transform(Number);

/** New ports after a conflict; both are needed, and they must differ. */
export const portFormSchema = z
  .object({ smtpPort: port, httpPort: port })
  .refine((values) => values.smtpPort !== values.httpPort, {
    error: "SMTP and HTTP need different ports",
    path: ["smtpPort"],
  });
