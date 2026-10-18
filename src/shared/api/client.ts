import { z } from "zod";

// Set once at startup, before the first render: the desktop app learns the API port
// from Tauri, the browser build talks to its own origin.
let apiBaseUrl = "";

/** Origin of the HTTP API. Empty means same origin. */
export function configureApiBaseUrl(url: string): void {
  apiBaseUrl = url.replace(/\/+$/, "");
}

export function apiUrl(path: string): string {
  return `${apiBaseUrl}${path}`;
}

export class ApiError extends Error {
  readonly status: number;
  /** The server's explanation (`{"error": "…"}`), safe to show to the user. */
  readonly detail: string | null;

  constructor(status: number, message: string, detail: string | null = null) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.detail = detail;
  }
}

const errorBodySchema = z.object({ error: z.string() });

async function readErrorDetail(response: Response): Promise<string | null> {
  try {
    const parsed = errorBodySchema.safeParse(await response.json());
    return parsed.success ? parsed.data.error : null;
  } catch {
    // Not every error response has a JSON body.
    return null;
  }
}

const HTTP_CLIENT_ERROR_MIN = 400;
const HTTP_SERVER_ERROR_MIN = 500;
const HTTP_NOT_FOUND = 404;

export function isClientError(error: unknown): boolean {
  return (
    error instanceof ApiError &&
    error.status >= HTTP_CLIENT_ERROR_MIN &&
    error.status < HTTP_SERVER_ERROR_MIN
  );
}

export function isNotFound(error: unknown): boolean {
  return error instanceof ApiError && error.status === HTTP_NOT_FOUND;
}

export async function request(path: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(apiUrl(path), init);
  if (!response.ok) {
    const method = init?.method ?? "GET";
    throw new ApiError(
      response.status,
      `${method} ${path} failed with status ${response.status}`,
      await readErrorDetail(response),
    );
  }
  return response;
}

export async function requestJson<T>(path: string, schema: z.ZodType<T>): Promise<T> {
  const response = await request(path);
  return schema.parse(await response.json());
}

/** Sends a JSON body; the content type also keeps the request out of reach of plain forms. */
export function sendJson(path: string, method: "POST" | "PUT", body: unknown): Promise<Response> {
  return request(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}
