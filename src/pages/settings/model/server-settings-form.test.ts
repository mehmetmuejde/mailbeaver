import { describe, expect, it } from "vitest";
import { serverSettingsFormSchema, toFormValues } from "./server-settings-form";

describe("server settings form", () => {
  const empty = { smtpPort: "", httpPort: "", maxMessages: "", maxMessageMb: "" };

  it("turns empty fields into defaults and numbers into numbers", () => {
    expect(serverSettingsFormSchema.parse(empty)).toEqual({
      smtpPort: null,
      httpPort: null,
      maxMessages: null,
      maxMessageMb: null,
    });
    expect(serverSettingsFormSchema.parse({ ...empty, smtpPort: " 2525 " }).smtpPort).toBe(2525);
  });

  it("rejects out-of-range values and equal ports", () => {
    expect(serverSettingsFormSchema.safeParse({ ...empty, smtpPort: "70000" }).success).toBe(false);
    expect(serverSettingsFormSchema.safeParse({ ...empty, maxMessageMb: "0" }).success).toBe(false);
    expect(serverSettingsFormSchema.safeParse({ ...empty, maxMessages: "1e3" }).success).toBe(
      false,
    );
    expect(serverSettingsFormSchema.safeParse({ ...empty, smtpPort: "8026" }).success).toBe(false);
  });

  it("shows saved values as text", () => {
    expect(
      toFormValues({ smtpPort: 2525, httpPort: null, maxMessages: 10, maxMessageMb: null }),
    ).toEqual({ smtpPort: "2525", httpPort: "", maxMessages: "10", maxMessageMb: "" });
  });
});
