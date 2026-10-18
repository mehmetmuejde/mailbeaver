import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createFakeMailApi, mailFixture, renderWithQuery } from "@/shared/lib/test";
import type { SettingsTab } from "../model/settings-search";
import { SettingsPage } from "./SettingsPage";

/** Stands in for the router: keeps the selected tab in memory. */
function Harness({ initial }: { initial: SettingsTab }) {
  const [tab, setTab] = useState(initial);
  return <SettingsPage tab={tab} onTabChange={setTab} />;
}

function setup(
  tab: SettingsTab = "general",
  messages = [mailFixture({ id: 1 }), mailFixture({ id: 2, folder: "trash" })],
) {
  const api = createFakeMailApi(messages);
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<Harness initial={tab} />);
  return { api, user };
}

describe("SettingsPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("shows where the servers listen", async () => {
    setup();
    expect(screen.getByRole("heading", { level: 1, name: "Settings" })).toBeInTheDocument();
    expect(await screen.findByText("127.0.0.1")).toBeInTheDocument();
    expect(screen.getByText("1026")).toBeInTheDocument();
    expect(screen.getByText("1.2.3")).toBeInTheDocument();
    expect(screen.getByText("/data/mailbeaver.db")).toBeInTheDocument();
    expect(screen.getByText("The newest 5000")).toBeInTheDocument();
    expect(screen.getByText("25.0 MB per message")).toBeInTheDocument();
  });

  it("shows one section per tab and switches with the keyboard", async () => {
    const { user } = setup();
    expect(screen.getByRole("tab", { name: "General" })).toHaveAttribute("aria-selected", "true");
    expect(screen.queryByRole("heading", { name: "Projects" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Projects" }));
    expect(await screen.findByRole("heading", { name: "Projects" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Server" })).not.toBeInTheDocument();

    await user.keyboard("{ArrowRight}");
    expect(screen.getByRole("tab", { name: "Trash" })).toHaveAttribute("aria-selected", "true");
  });

  it("turns on the update check and reports a newer release", async () => {
    const { api, user } = setup();
    const toggle = await screen.findByRole("switch", { name: "Check for updates" });
    await waitFor(() => expect(toggle).toBeEnabled());
    await user.click(toggle);

    await waitFor(() => expect(api.settings().updateCheck).toBe(true));
    expect(await screen.findByText(/Version 1.3.0 is available/)).toBeInTheDocument();
  });

  it("pauses capture from the general tab", async () => {
    const { api, user } = setup();
    const toggle = await screen.findByRole("switch", { name: "Pause capture" });
    await waitFor(() => expect(toggle).toBeEnabled());
    await user.click(toggle);
    await waitFor(() => expect(api.capture().paused).toBe(true));
    expect(toggle).toBeChecked();
  });

  it("shows runtime statistics and refreshes them", async () => {
    const { api, user } = setup("stats");
    expect(await screen.findByText("96.0 KB")).toBeInTheDocument();
    expect(screen.getByText("2 messages (4.0 KB)")).toBeInTheDocument();
    expect(screen.getByText("1 message")).toBeInTheDocument();

    const before = api.calls("GET", "/api/stats").length;
    await user.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(api.calls("GET", "/api/stats").length).toBeGreaterThan(before));
  });

  it("explains how tools and agents use the API", async () => {
    const { user } = setup("api");
    const writeText = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined);
    expect(
      await screen.findByRole("link", { name: "http://127.0.0.1:8026/api/openapi.json" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Copy MCP command" }));
    expect(writeText).toHaveBeenCalledWith(
      "claude mcp add --transport http mailbeaver http://127.0.0.1:8026/mcp",
    );
    writeText.mockRestore();
  });

  it("switches the theme from the general tab", async () => {
    const { user } = setup();
    await user.selectOptions(screen.getByRole("combobox", { name: "Theme" }), "Dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    await user.selectOptions(screen.getByRole("combobox", { name: "Theme" }), "Light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("saves the trash retention on change", async () => {
    const { api, user } = setup("trash");
    const select = screen.getByRole("combobox", { name: "Empty trash automatically" });
    await waitFor(() => expect(select).toBeEnabled());
    expect(select).toHaveValue("off");

    await user.selectOptions(select, "After 30 days");

    expect(await screen.findByText("Saved.")).toBeInTheDocument();
    expect(api.settings()).toMatchObject({ trashRetentionDays: 30, trayGrouping: "all" });

    await user.selectOptions(select, "Never");
    await waitFor(() =>
      expect(api.settings()).toMatchObject({ trashRetentionDays: null, trayGrouping: "all" }),
    );
  });

  it("saves how the tray groups messages", async () => {
    const { api, user } = setup("tray");
    const select = screen.getByRole("combobox", { name: "Group messages" });
    await waitFor(() => expect(select).toBeEnabled());

    await user.selectOptions(select, "By project");

    await waitFor(() =>
      expect(api.settings()).toMatchObject({ trashRetentionDays: null, trayGrouping: "project" }),
    );
  });

  it("deletes all messages only after confirmation", async () => {
    const { api, user } = setup("danger");
    const trigger = screen.getByRole("button", { name: "Delete all messages" });
    await waitFor(() => expect(trigger).toBeEnabled());

    await user.click(trigger);
    expect(screen.getByRole("alertdialog")).toHaveTextContent("All 2 messages");
    await user.click(screen.getByRole("button", { name: "Delete all" }));

    expect(await screen.findByText("All messages were deleted.")).toBeInTheDocument();
    expect(api.messages()).toEqual([]);
  });

  it("resets everything only after confirmation", async () => {
    const { api, user } = setup("danger");

    await user.click(screen.getByRole("button", { name: "Reset Mailbeaver" }));
    expect(screen.getByRole("alertdialog")).toHaveTextContent("projects and settings");
    await user.click(screen.getByRole("button", { name: "Reset everything" }));

    expect(await screen.findByText("Mailbeaver was reset.")).toBeInTheDocument();
    expect(api.messages()).toEqual([]);
    expect(api.calls("DELETE", "/api/data")).toHaveLength(1);
  });

  it("reports failures", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 500 })),
    );
    const user = userEvent.setup();
    renderWithQuery(<Harness initial="general" />);

    expect(await screen.findByText("Server information could not be loaded.")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Trash" }));
    expect(await screen.findByText("Settings could not be loaded.")).toBeInTheDocument();
  });
});
