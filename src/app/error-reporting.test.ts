import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError } from "@/shared/api";
import { describeError, installErrorReporting, reportRequestError } from "./error-reporting";

function sentBodies(fetchMock: ReturnType<typeof vi.fn>): unknown[] {
  return fetchMock.mock.calls.map(([, init]) => JSON.parse(String(init?.body)));
}

describe("error reporting", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("describes what was thrown", () => {
    expect(describeError(new ApiError(409, "POST /api/projects failed", "name taken"))).toBe(
      "POST /api/projects failed: name taken",
    );
    expect(describeError(new ApiError(500, "GET /api/x failed"))).toBe("GET /api/x failed");
    expect(describeError(new Error("boom"))).toBe("boom");
    expect(describeError("text")).toBe("text");
  });

  it("reports client errors as warnings and others as errors", () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetchMock);
    reportRequestError("mutation", new ApiError(422, "PUT /api/settings failed"));
    reportRequestError("query", new ApiError(503, "GET /api/folders failed"), '["folders"]');

    expect(sentBodies(fetchMock)).toEqual([
      { level: "warn", target: "mutation", message: "PUT /api/settings failed" },
      { level: "error", target: "query", message: '["folders"]: GET /api/folders failed' },
    ]);
  });

  it("reports uncaught errors and rejections of the page", () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetchMock);
    const target = new EventTarget() as Window;
    installErrorReporting(target);

    target.dispatchEvent(
      Object.assign(new Event("error"), {
        message: "x is undefined",
        filename: "app.js",
        lineno: 3,
      }),
    );
    target.dispatchEvent(
      Object.assign(new Event("unhandledrejection"), { reason: new Error("lost") }),
    );

    expect(sentBodies(fetchMock)).toEqual([
      { level: "error", target: "window", message: "x is undefined (app.js:3)" },
      { level: "error", target: "promise", message: "lost" },
    ]);
  });
});
