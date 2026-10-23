import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { z } from "zod";

/** Sent by the desktop host when a message is picked in the tray menu. */
const OPEN_MESSAGE_EVENT = "open-message";
const openMessageSchema = z.object({ id: z.number().int().positive() });

/** True inside the Tauri desktop app, false in a browser. */
export function isDesktopApp(): boolean {
  return isTauri();
}

/** macOS uses ⌘ where Windows and Linux use Ctrl, e.g. to add to a selection. */
export function isMacPlatform(): boolean {
  return /Mac/.test(window.navigator.userAgent);
}

/**
 * True when the macOS window draws its traffic lights over the page (overlay title
 * bar); the UI then leaves room for them and acts as the title bar.
 */
export function hasOverlayTitleBar(): boolean {
  return isTauri() && /Mac/.test(window.navigator.userAgent);
}

/** Starts the desktop app again, e.g. so new ports apply. */
export function restartApp(): Promise<void> {
  return invoke("restart_app");
}

/** Saves ports while the servers are down (desktop app only); restart afterwards. */
export function saveServerPorts(smtpPort: number, httpPort: number): Promise<void> {
  return invoke("save_server_ports", { smtpPort, httpPort });
}

/** Shows a sample system notification (desktop app only). */
export function sendTestNotification(sound: boolean): Promise<void> {
  return invoke("send_test_notification", { sound });
}

/**
 * Saves the file behind `url`: the desktop webview has no download manager, so the
 * system browser fetches it there; a browser downloads it directly.
 */
export async function downloadFile(url: string): Promise<void> {
  if (isTauri()) {
    await openUrl(url);
    return;
  }
  const link = document.createElement("a");
  link.href = url;
  link.download = "";
  link.click();
}

/** Opens a URL in the system's default application (desktop app only). */
export function openExternalUrl(url: string): Promise<void> {
  return openUrl(url);
}

/**
 * Calls `handler` with the message id whenever the tray asks to show a message.
 * Returns the function that stops listening; in a browser nothing is ever sent.
 */
export async function onOpenMessageRequest(handler: (id: number) => void): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen(OPEN_MESSAGE_EVENT, (event) => {
    const payload = openMessageSchema.safeParse(event.payload);
    if (payload.success) {
      handler(payload.data.id);
    }
  });
}
