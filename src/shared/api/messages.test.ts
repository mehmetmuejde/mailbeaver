import { afterEach, describe, expect, it, vi } from "vitest";
import { ZodError } from "zod";
import { ApiError, configureApiBaseUrl, isClientError, isNotFound } from "./client";
import { attachmentUrl, fetchMessages, moveMessage } from "./messages";

const page = { total: 0, unread: 0, messages: [] };

describe("messages API", () => {
  afterEach(() => {
    configureApiBaseUrl("");
    vi.unstubAllGlobals();
  });

  it("prefixes the configured base URL without a double slash", async () => {
    const fetchMock = vi.fn(async () => Response.json(page));
    vi.stubGlobal("fetch", fetchMock);
    configureApiBaseUrl("http://127.0.0.1:8026/");

    await expect(fetchMessages({ folder: "inbox", limit: 50 })).resolves.toEqual(page);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:8026/api/messages?limit=50&folder=inbox",
      undefined,
    );
    expect(attachmentUrl(3, 1)).toBe("http://127.0.0.1:8026/api/messages/3/attachments/1");
  });

  it("sends the mailbox filter and a trimmed search, and skips an empty search", async () => {
    const fetchMock = vi.fn(async () => Response.json(page));
    vi.stubGlobal("fetch", fetchMock);

    await fetchMessages({ folder: "trash", project: "none", q: "  reset ", limit: 10 });
    await fetchMessages({ folder: "trash", project: 3, q: "   ", limit: 10 });
    await fetchMessages({ q: "reset", limit: 10 });

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/api/messages?limit=10&folder=trash&project=none&q=reset",
      undefined,
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/api/messages?limit=10&folder=trash&project=3",
      undefined,
    );
    // Without a folder the search covers all of them.
    expect(fetchMock).toHaveBeenNthCalledWith(3, "/api/messages?limit=10&q=reset", undefined);
  });

  it("moves a message with a JSON body", async () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetchMock);

    await moveMessage({ id: 4, folder: "archive" });

    expect(fetchMock).toHaveBeenCalledWith("/api/messages/4/move", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: '{"folder":"archive"}',
    });
  });

  it("rejects responses that do not match the schema", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ total: "many" })),
    );
    await expect(fetchMessages({ folder: "inbox", limit: 50 })).rejects.toBeInstanceOf(ZodError);
  });

  it("keeps the server's explanation of an error", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ error: "name taken" }, { status: 409 })),
    );
    const error = await fetchMessages({ folder: "inbox", limit: 50 }).catch(
      (caught: unknown) => caught,
    );
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).detail).toBe("name taken");
  });

  it("turns error statuses into ApiError", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 404 })),
    );
    const error = await fetchMessages({ folder: "inbox", limit: 50 }).catch(
      (caught: unknown) => caught,
    );

    expect(error).toBeInstanceOf(ApiError);
    expect(isNotFound(error)).toBe(true);
    expect(isClientError(error)).toBe(true);
    expect(isClientError(new ApiError(500, "boom"))).toBe(false);
  });
});
