import { afterEach, describe, expect, it, vi } from "vitest";
import { ZodError } from "zod";
import { updateSettings } from "./settings";

describe("settings API", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("saves the settings and returns what the server stored", async () => {
    const stored = {
      trashRetentionDays: 30,
      trayGrouping: "project",
      theme: "dark",
      sidebarCollapsed: true,
      notifications: "projects",
      notificationSound: true,
      unreadBadge: false,
      updateCheck: true,
    } as const;
    const fetchMock = vi.fn(async () => Response.json(stored));
    vi.stubGlobal("fetch", fetchMock);

    await expect(updateSettings(stored)).resolves.toEqual(stored);
    expect(fetchMock).toHaveBeenCalledWith("/api/settings", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: '{"trashRetentionDays":30,"trayGrouping":"project","theme":"dark","sidebarCollapsed":true,"notifications":"projects","notificationSound":true,"unreadBadge":false,"updateCheck":true}',
    });
  });

  it("rejects a retention outside the allowed range", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        Response.json({
          trashRetentionDays: 0,
          trayGrouping: "all",
          theme: "system",
          sidebarCollapsed: false,
        }),
      ),
    );
    await expect(
      updateSettings({
        trashRetentionDays: null,
        trayGrouping: "all",
        theme: "system",
        sidebarCollapsed: false,
        notifications: "all",
        notificationSound: false,
        unreadBadge: true,
        updateCheck: false,
      }),
    ).rejects.toBeInstanceOf(ZodError);
  });
});
