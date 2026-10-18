import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  downloadFile,
  isDesktopApp,
  onOpenMessageRequest,
  openExternalUrl,
  restartApp,
  saveServerPorts,
  sendTestNotification,
} from "./platform";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => undefined),
  isTauri: vi.fn(() => true),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(async () => undefined) }));

type Listener = (event: { payload: unknown }) => void;

describe("platform", () => {
  afterEach(() => {
    vi.mocked(isTauri).mockReturnValue(true);
  });

  it("delegates to Tauri", async () => {
    expect(isDesktopApp()).toBe(true);
    expect(isTauri).toHaveBeenCalled();

    await openExternalUrl("http://127.0.0.1:8026/api/messages/1/attachments/0");
    expect(openUrl).toHaveBeenCalledWith("http://127.0.0.1:8026/api/messages/1/attachments/0");

    await sendTestNotification(true);
    expect(invoke).toHaveBeenCalledWith("send_test_notification", { sound: true });
    await saveServerPorts(2525, 8080);
    expect(invoke).toHaveBeenCalledWith("save_server_ports", { smtpPort: 2525, httpPort: 8080 });
    await restartApp();
    expect(invoke).toHaveBeenCalledWith("restart_app");
  });

  it("passes valid open-message requests from the tray to the handler", async () => {
    let listener: Listener = () => undefined;
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      listener = handler as Listener;
      return unlisten;
    });
    const handler = vi.fn();

    const stop = await onOpenMessageRequest(handler);
    listener({ payload: { id: 4 } });
    listener({ payload: { id: "4" } });

    expect(listen).toHaveBeenCalledWith("open-message", expect.any(Function));
    expect(handler).toHaveBeenCalledExactlyOnceWith(4);
    stop();
    expect(unlisten).toHaveBeenCalled();
  });

  it("downloads through the system browser in the desktop app and directly otherwise", async () => {
    await downloadFile("http://127.0.0.1:8026/api/messages/1/raw?download=true");
    expect(openUrl).toHaveBeenCalledWith("http://127.0.0.1:8026/api/messages/1/raw?download=true");

    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(openUrl).mockClear();
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => undefined);
    await downloadFile("/api/messages/1/raw?download=true");
    expect(click).toHaveBeenCalled();
    expect(openUrl).not.toHaveBeenCalled();
    click.mockRestore();
  });

  it("listens to nothing in a browser", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(listen).mockClear();
    const stop = await onOpenMessageRequest(vi.fn());
    expect(listen).not.toHaveBeenCalled();
    expect(stop()).toBeUndefined();
  });
});
