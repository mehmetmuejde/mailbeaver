import { createMemoryHistory } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { screen, waitFor } from "@testing-library/react";
import { act } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "@/app/App";
import {
  createFakeMailApi,
  FakeEventSource,
  mailFixture,
  renderWithQuery,
} from "@/shared/lib/test";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: vi.fn(() => true), invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

type Listener = (event: { payload: unknown }) => void;

describe("open-message requests from the tray", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("opens the requested message in the inbox", async () => {
    let listener: Listener = () => undefined;
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      listener = handler as Listener;
      return () => undefined;
    });
    vi.stubGlobal("EventSource", FakeEventSource);
    const api = createFakeMailApi([mailFixture({ id: 5, subject: "From the tray" })]);
    vi.stubGlobal("fetch", api.fetchMock);
    const history = createMemoryHistory({ initialEntries: ["/settings"] });
    renderWithQuery(
      <App status={{ apiBaseUrl: "", smtpAddress: null, error: null }} history={history} />,
    );
    await waitFor(() => expect(listen).toHaveBeenCalled());

    act(() => listener({ payload: { id: 5 } }));

    expect(
      await screen.findByRole("heading", { level: 2, name: "From the tray" }),
    ).toBeInTheDocument();
    expect(history.location.pathname).toBe("/inbox");
  });

  it("stops listening when the app unmounts before the listener is ready", async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async () => unlisten);
    vi.stubGlobal("EventSource", FakeEventSource);
    vi.stubGlobal("fetch", createFakeMailApi([]).fetchMock);
    const { unmount } = renderWithQuery(
      <App
        status={{ apiBaseUrl: "", smtpAddress: null, error: null }}
        history={createMemoryHistory({ initialEntries: ["/inbox"] })}
      />,
    );
    await screen.findByRole("heading", { level: 1, name: "Inbox" });
    unmount();
    await waitFor(() => expect(unlisten).toHaveBeenCalled());
  });
});
