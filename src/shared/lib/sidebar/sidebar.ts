import { useSyncExternalStore } from "react";

// A layout preference of this device: browser storage is enough.
const STORAGE_KEY = "mailbeaver.sidebar-collapsed";

function readCollapsed(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

let collapsed = readCollapsed();
const listeners = new Set<() => void>();

export function setSidebarCollapsed(value: boolean): void {
  collapsed = value;
  try {
    window.localStorage.setItem(STORAGE_KEY, String(value));
  } catch {
    // Without storage the choice lasts until the app reloads.
  }
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useSidebarCollapsed(): boolean {
  return useSyncExternalStore(subscribe, () => collapsed);
}
