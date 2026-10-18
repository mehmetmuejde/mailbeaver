import { invoke, isTauri } from "@tauri-apps/api/core";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createFakeMailApi, renderWithQuery } from "@/shared/lib/test";
import { SettingsNotifications } from "./SettingsNotifications";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => undefined),
  isTauri: vi.fn(() => false),
}));

function setup() {
  const api = createFakeMailApi([]);
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<SettingsNotifications />);
  return { api, user };
}

describe("SettingsNotifications", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.mocked(isTauri).mockReturnValue(false);
  });

  it("saves mode, sound and badge", async () => {
    const { api, user } = setup();
    const mode = await screen.findByRole("combobox", { name: "Notify about" });
    await waitFor(() => expect(mode).toBeEnabled());

    await user.selectOptions(mode, "projects");
    await waitFor(() => expect(api.settings().notifications).toBe("projects"));
    expect(screen.getByText(/Choose the projects under Projects/)).toBeInTheDocument();

    await user.click(screen.getByRole("switch", { name: "Play a sound" }));
    await waitFor(() => expect(api.settings().notificationSound).toBe(true));

    await user.click(screen.getByRole("switch", { name: "Unread count on the app icon" }));
    await waitFor(() => expect(api.settings().unreadBadge).toBe(false));
  });

  it("turns the sound off together with the notifications", async () => {
    const { user } = setup();
    const mode = await screen.findByRole("combobox", { name: "Notify about" });
    await waitFor(() => expect(mode).toBeEnabled());
    await user.selectOptions(mode, "off");
    await waitFor(() =>
      expect(screen.getByRole("switch", { name: "Play a sound" })).toBeDisabled(),
    );
  });

  it("sends a test notification in the desktop app only", async () => {
    expect(isTauri()).toBe(false);
    setup();
    await screen.findByRole("combobox", { name: "Notify about" });
    expect(screen.queryByRole("button", { name: "Send test notification" })).toBeNull();
  });

  it("sends a test notification with the saved sound choice", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    const { user } = setup();
    const button = await screen.findByRole("button", { name: "Send test notification" });
    await waitFor(() =>
      expect(screen.getByRole("combobox", { name: "Notify about" })).toBeEnabled(),
    );
    await user.click(button);
    expect(invoke).toHaveBeenCalledWith("send_test_notification", { sound: false });
  });
});
