import { afterEach, describe, expect, it, vi } from "vitest";
import { applyTheme, initTheme, readThemePreference, setThemePreference } from "./theme";

function stubSystemDark(dark: boolean) {
  const listeners: (() => void)[] = [];
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: dark,
      addEventListener: (_: string, listener: () => void) => listeners.push(listener),
    })),
  );
  return listeners;
}

describe("theme", () => {
  afterEach(() => {
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("follows the system until the user picks a theme", () => {
    stubSystemDark(true);
    expect(readThemePreference()).toBe("system");
    applyTheme("system");
    expect(document.documentElement.dataset.theme).toBe("dark");

    setThemePreference("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(readThemePreference()).toBe("light");
  });

  it("ignores unknown stored values", () => {
    window.localStorage.setItem("mailbeaver.theme", "purple");
    expect(readThemePreference()).toBe("system");
  });

  it("re-applies the system theme when it changes", () => {
    const listeners = stubSystemDark(false);
    setThemePreference("system");
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("light");

    stubSystemDark(true);
    for (const listener of listeners) {
      listener();
    }
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
