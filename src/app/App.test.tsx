import { createMemoryHistory } from "@tanstack/react-router";
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createFakeMailApi,
  FakeEventSource,
  mailFixture,
  renderWithQuery,
} from "@/shared/lib/test";
import { App } from "./App";

const RUNNING = { apiBaseUrl: "", smtpAddress: "127.0.0.1:1026", error: null };

function renderApp(path = "/") {
  const history = createMemoryHistory({ initialEntries: [path] });
  renderWithQuery(<App status={RUNNING} history={history} />);
  return history;
}

describe("App", () => {
  let api: ReturnType<typeof createFakeMailApi>;

  beforeEach(() => {
    FakeEventSource.instances = [];
    vi.stubGlobal("EventSource", FakeEventSource);
    api = createFakeMailApi([
      mailFixture({ id: 1, subject: "Welcome" }),
      mailFixture({ id: 2, subject: "Old", folder: "trash", isRead: true }),
    ]);
    vi.stubGlobal("fetch", api.fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("explains a startup failure instead of showing the inbox", () => {
    renderWithQuery(
      <App
        status={{ apiBaseUrl: null, smtpAddress: null, error: "SMTP port 1026 is not available" }}
      />,
    );

    expect(screen.getByRole("heading", { name: "Mailbeaver could not start" })).toBeVisible();
    expect(screen.getByRole("alert")).toHaveTextContent("SMTP port 1026 is not available");
    expect(screen.getByText(/Another program already uses the port/)).toBeInTheDocument();
  });

  it("explains a database written by a newer version", () => {
    renderWithQuery(
      <App
        status={{
          apiBaseUrl: null,
          smtpAddress: null,
          error:
            "cannot open database /x.db: database schema version 3 is newer than this build supports",
        }}
      />,
    );

    expect(screen.getByText(/written by a newer Mailbeaver/)).toBeInTheDocument();
    expect(screen.queryByText(/uses the port/)).not.toBeInTheDocument();
  });

  it("links a newer release below the version", async () => {
    const settings = (await (await api.fetchMock("/api/settings")).json()) as object;
    await api.fetchMock("/api/settings", {
      method: "PUT",
      body: JSON.stringify({ ...settings, updateCheck: true }),
    });
    renderApp();

    expect(await screen.findByRole("link", { name: "Version 1.3.0 available" })).toHaveAttribute(
      "href",
      "https://github.com/mehmetmuejde/mailbeaver/releases/tag/v1.3.0",
    );
  });

  it("shows while capture is paused and resumes it", async () => {
    await api.fetchMock("/api/capture", { method: "PUT", body: JSON.stringify({ paused: true }) });
    const user = userEvent.setup();
    renderApp();

    expect(await screen.findByText(/Capture is paused/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Resume capture" }));

    await waitFor(() => expect(screen.queryByText(/Capture is paused/)).not.toBeInTheDocument());
    expect(api.capture().paused).toBe(false);
  });

  it("opens the inbox and shows folder counts in the navigation", async () => {
    const history = renderApp();

    expect(await screen.findByRole("heading", { level: 1, name: "Inbox" })).toBeVisible();
    expect(history.location.pathname).toBe("/inbox");
    const nav = within(screen.getByRole("navigation", { name: "Folders" }));
    expect(await nav.findByRole("link", { name: "Inbox 1 unread 1 message" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(await nav.findByRole("link", { name: "Trash 1 message" })).toBeInTheDocument();
    expect(document.title).toBe("Inbox – Mailbeaver");
    expect(await screen.findByText("Version 1.2.3")).toBeInTheDocument();
  });

  it("navigates between views and moves focus to the new page", async () => {
    const user = userEvent.setup();
    renderApp("/inbox");
    const nav = within(await screen.findByRole("navigation", { name: "Folders" }));

    await user.click(nav.getByRole("link", { name: /Settings/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Settings" })).toBeVisible();
    expect(screen.getByRole("main")).toHaveFocus();

    await user.click(nav.getByRole("link", { name: /Trash/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Trash" })).toBeVisible();
    expect(await screen.findByRole("button", { name: /Old/ })).toBeInTheDocument();
  });

  it("keeps the open message in the URL", async () => {
    const user = userEvent.setup();
    const history = renderApp("/inbox");

    await user.click(await screen.findByRole("button", { name: /Welcome/ }));
    await waitFor(() => expect(history.location.search).toContain("message=1"));
  });

  it("searches from the page header and keeps the query in the URL", async () => {
    const user = userEvent.setup();
    const history = renderApp("/trash");
    await screen.findByRole("heading", { level: 1, name: "Trash" });
    const field = screen.getByRole("searchbox", { name: "Search all mail" });

    await user.type(field, "welc");

    await waitFor(() => expect(history.location.search).toContain("q=welc"));
    expect(history.location.pathname).toBe("/trash");
    expect(field).toHaveFocus();
    expect(await screen.findByRole("button", { name: /Welcome/ })).toHaveTextContent("Inbox");

    await user.keyboard("{Escape}");
    await waitFor(() => expect(history.location.search).not.toContain("q="));
  });

  it("marks a whole folder read from the sidebar menu", async () => {
    const user = userEvent.setup();
    renderApp("/inbox");
    const nav = within(await screen.findByRole("navigation", { name: "Folders" }));
    const inbox = await nav.findByRole("link", { name: "Inbox 1 unread 1 message" });

    fireEvent.contextMenu(inbox);
    await user.click(await screen.findByRole("menuitem", { name: "Mark all as read" }));

    await waitFor(() => expect(api.messages().every((message) => message.isRead)).toBe(true));
    expect(api.calls("POST", "/api/messages/mark-read")).toHaveLength(1);
  });

  it("remembers each folder's view and restores it on the next visit", async () => {
    const user = userEvent.setup();
    const history = renderApp("/inbox");
    await screen.findByRole("button", { name: /Welcome/ });

    await user.click(screen.getByRole("radio", { name: "Sender" }));
    await waitFor(() => expect(api.views().inbox.grouping).toBe("sender"));
    expect(api.views().archive.grouping).toBe("all");

    const nav = within(screen.getByRole("navigation", { name: "Folders" }));
    await user.click(nav.getByRole("link", { name: /Archive/ }));
    await screen.findByRole("heading", { level: 1, name: "Archive" });
    // Archive keeps its own (default) view.
    expect(history.location.search).not.toContain("view=");

    await user.click(nav.getByRole("link", { name: /Inbox/ }));
    await waitFor(() => expect(history.location.search).toContain("view=sender"));
    expect(await screen.findByRole("radio", { name: "Sender" })).toBeChecked();
  });

  it("keeps the sidebar state and the theme in the settings", async () => {
    const user = userEvent.setup();
    renderApp("/inbox");
    await screen.findByRole("heading", { level: 1, name: "Inbox" });

    await user.click(screen.getByRole("button", { name: "Hide sidebar" }));
    await waitFor(() => expect(api.settings().sidebarCollapsed).toBe(true));
    await user.click(screen.getByRole("button", { name: "Show sidebar" }));
    await waitFor(() => expect(api.settings().sidebarCollapsed).toBe(false));
  });

  it("has no search on the settings page", async () => {
    renderApp("/settings");
    await screen.findByRole("heading", { level: 1, name: "Settings" });
    expect(screen.queryByRole("searchbox")).not.toBeInTheDocument();
  });

  it("slides the sidebar out and back in", async () => {
    const user = userEvent.setup();
    renderApp("/inbox");
    await screen.findByRole("heading", { level: 1, name: "Inbox" });

    await user.click(screen.getByRole("button", { name: "Hide sidebar" }));
    const show = await screen.findByRole("button", { name: "Show sidebar" });
    expect(show).toHaveAttribute("aria-expanded", "false");
    // Hidden sidebars are inert: no focus, not read out.
    expect(document.getElementById("app-sidebar")).toHaveAttribute("inert");

    await user.click(show);
    expect(document.getElementById("app-sidebar")).not.toHaveAttribute("inert");
    expect(screen.getByRole("button", { name: "Hide sidebar" })).toHaveAttribute(
      "aria-expanded",
      "true",
    );
  });

  it("refreshes the lists when the server announces a change", async () => {
    renderApp("/inbox");
    await screen.findByRole("button", { name: /Welcome/ });
    const listCalls = () => api.calls("GET", "/api/messages").length;
    const before = listCalls();

    const source = FakeEventSource.instances[0];
    expect(source?.url).toBe("/api/events");
    act(() => source?.emit("change"));

    await waitFor(() => expect(listCalls()).toBeGreaterThan(before));
  });

  it("keeps the settings tab in the URL", async () => {
    const user = userEvent.setup();
    const history = renderApp("/settings?tab=projects");

    expect(await screen.findByRole("tab", { name: "Projects" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await user.click(screen.getByRole("tab", { name: "Tray menu" }));
    await waitFor(() => expect(history.location.search).toContain("tab=tray"));
    await user.click(screen.getByRole("tab", { name: "General" }));
    await waitFor(() => expect(history.location.search).not.toContain("tab="));
  });

  it("shows a not-found page for unknown paths", async () => {
    renderApp("/nowhere");
    expect(await screen.findByRole("heading", { name: "Page not found" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Go to the inbox" })).toBeInTheDocument();
  });
});
