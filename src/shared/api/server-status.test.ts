import { invoke, isTauri } from "@tauri-apps/api/core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ZodError } from "zod";
import { fetchServerStatus } from "./server-status";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));

describe("fetchServerStatus", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  it("uses the same origin in a browser", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    await expect(fetchServerStatus()).resolves.toEqual({
      apiBaseUrl: "",
      smtpAddress: null,
      error: null,
    });
    expect(invoke).not.toHaveBeenCalled();
  });

  it("asks the desktop host and validates the answer", async () => {
    const status = {
      apiBaseUrl: "http://127.0.0.1:8026",
      smtpAddress: "127.0.0.1:1026",
      error: null,
    };
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue(status);

    await expect(fetchServerStatus()).resolves.toEqual(status);
    expect(invoke).toHaveBeenCalledWith("server_status");
  });

  it("rejects a malformed answer", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({ apiBaseUrl: 8026 });
    await expect(fetchServerStatus()).rejects.toBeInstanceOf(ZodError);
  });
});
