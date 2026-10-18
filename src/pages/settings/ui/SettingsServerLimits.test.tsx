import { invoke } from "@tauri-apps/api/core";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createFakeMailApi, renderWithQuery } from "@/shared/lib/test";
import { SettingsServerLimits } from "./SettingsServerLimits";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => undefined),
  isTauri: vi.fn(() => true),
}));

function setup() {
  const api = createFakeMailApi([]);
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<SettingsServerLimits />);
  return { api, user };
}

describe("SettingsServerLimits", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("locks values set by an environment variable", async () => {
    setup();
    const http = await screen.findByRole("textbox", { name: "HTTP port" });
    expect(http).toBeDisabled();
    expect(http).toHaveAccessibleDescription("Set by MAILBEAVER_HTTP_PORT");
    expect(screen.getByRole("textbox", { name: "SMTP port" })).toHaveAttribute(
      "placeholder",
      "1026",
    );
  });

  it("validates, saves and offers a restart", async () => {
    const { api, user } = setup();
    const smtp = await screen.findByRole("textbox", { name: "SMTP port" });

    await user.type(smtp, "8026");
    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("SMTP and HTTP need different ports")).toBeInTheDocument();

    await user.clear(smtp);
    await user.type(smtp, "2525");
    await user.type(screen.getByRole("textbox", { name: "Largest message" }), "10");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(api.serverSettings()).toEqual({
        smtpPort: 2525,
        httpPort: null,
        maxMessages: null,
        maxMessageMb: 10,
      }),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Saved. Restart Mailbeaver to apply.");
    await user.click(screen.getByRole("button", { name: "Restart now" }));
    expect(invoke).toHaveBeenCalledWith("restart_app");
  });
});
