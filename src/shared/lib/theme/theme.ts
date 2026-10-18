import { useSyncExternalStore } from "react";
import { z } from "zod";
import { THEMES } from "@/shared/api";

// One list of themes for the stored setting and this local copy.
export const THEME_PREFERENCES = THEMES;
export type ThemePreference = (typeof THEME_PREFERENCES)[number];

// A display preference of this device, not user data: browser storage is enough.
const STORAGE_KEY = "mailbeaver.theme";
const DARK_QUERY = "(prefers-color-scheme: dark)";
const preferenceSchema = z.enum(THEME_PREFERENCES);

export function readThemePreference(): ThemePreference {
  try {
    const stored = preferenceSchema.safeParse(window.localStorage.getItem(STORAGE_KEY));
    return stored.success ? stored.data : "system";
  } catch {
    // Storage can be blocked (private mode); the system theme is a safe default.
    return "system";
  }
}

function systemPrefersDark(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia(DARK_QUERY).matches;
}

/** Sets `data-theme` on `<html>`, which the stylesheet's dark variant keys on. */
export function applyTheme(preference: ThemePreference): void {
  const dark = preference === "dark" || (preference === "system" && systemPrefersDark());
  document.documentElement.dataset.theme = dark ? "dark" : "light";
}

let current: ThemePreference = readThemePreference();
const listeners = new Set<() => void>();

export function setThemePreference(preference: ThemePreference): void {
  current = preference;
  try {
    window.localStorage.setItem(STORAGE_KEY, preference);
  } catch {
    // Without storage the choice lasts until the app reloads.
  }
  applyTheme(preference);
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useThemePreference(): ThemePreference {
  return useSyncExternalStore(subscribe, () => current);
}

/** Applies the stored theme now and follows the system while "system" is chosen. */
export function initTheme(): void {
  applyTheme(current);
  if (typeof window.matchMedia === "function") {
    window.matchMedia(DARK_QUERY).addEventListener("change", () => applyTheme(current));
  }
}
