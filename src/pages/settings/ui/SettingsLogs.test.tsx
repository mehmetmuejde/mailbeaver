import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { LogEntry } from "@/shared/api";
import { createFakeMailApi, renderWithQuery } from "@/shared/lib/test";
import { SettingsLogs } from "./SettingsLogs";

const ENTRIES: LogEntry[] = [
  {
    id: 1,
    createdAt: 0,
    level: "info",
    source: "backend",
    target: "mailbeaver_core::smtp",
    message: "message received id=1 from=a@shop.test",
  },
  {
    id: 2,
    createdAt: 1000,
    level: "error",
    source: "frontend",
    target: "query",
    message: "GET /api/folders failed with status 500",
  },
];

function setup() {
  const api = createFakeMailApi([], [], ENTRIES);
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<SettingsLogs />);
  return { api, user };
}

function rows() {
  return screen.getAllByRole("row").slice(1);
}

function firstRow(): HTMLElement {
  const [row] = rows();
  if (row === undefined) {
    throw new Error("no log rows");
  }
  return row;
}

describe("SettingsLogs", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("lists entries newest first and filters them", async () => {
    const { user } = setup();
    expect(await screen.findByText(/GET \/api\/folders failed/)).toBeInTheDocument();
    expect(rows()).toHaveLength(2);
    expect(within(firstRow()).getByText("UI")).toBeInTheDocument();

    await user.selectOptions(screen.getByRole("combobox", { name: "Level" }), "error");
    await waitFor(() => expect(rows()).toHaveLength(1));

    await user.selectOptions(screen.getByRole("combobox", { name: "Level" }), "info");
    await user.selectOptions(screen.getByRole("combobox", { name: "Source" }), "backend");
    await waitFor(() => expect(rows()).toHaveLength(1));
    expect(within(firstRow()).getByText(/message received/)).toBeInTheDocument();
  });

  it("copies the entries as text", async () => {
    const { user } = setup();
    const writeText = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined);
    await screen.findByText(/GET \/api\/folders failed/);

    await user.click(screen.getByRole("button", { name: "Copy" }));
    expect(writeText).toHaveBeenCalledWith(
      "1970-01-01T00:00:01.000Z ERROR [frontend] query: GET /api/folders failed with status 500\n" +
        "1970-01-01T00:00:00.000Z INFO [backend] mailbeaver_core::smtp: message received id=1 from=a@shop.test",
    );
    writeText.mockRestore();
  });

  it("clears the log after asking", async () => {
    const { api, user } = setup();
    await screen.findByText(/GET \/api\/folders failed/);

    await user.click(screen.getByRole("button", { name: "Clear log" }));
    await user.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "Clear log" }),
    );

    expect(await screen.findByText("No log entries.")).toBeInTheDocument();
    expect(api.logs()).toEqual([]);
  });
});
