import { ApiError, isClientError, reportLog } from "@/shared/api";

/** Short text of anything thrown, with the server's explanation when there is one. */
export function describeError(error: unknown): string {
  if (error instanceof ApiError) {
    return error.detail === null ? error.message : `${error.message}: ${error.detail}`;
  }
  return error instanceof Error ? error.message : String(error);
}

/** A failed request; client errors (4xx) are warnings, everything else an error. */
export function reportRequestError(
  target: "query" | "mutation",
  error: unknown,
  context = "",
): void {
  const prefix = context === "" ? "" : `${context}: `;
  reportLog(isClientError(error) ? "warn" : "error", target, `${prefix}${describeError(error)}`);
}

/** Uncaught errors and unhandled promise rejections of the page go to the server log. */
export function installErrorReporting(target: Window = window): void {
  target.addEventListener("error", (event) => {
    const location = event.filename === "" ? "" : ` (${event.filename}:${event.lineno})`;
    reportLog("error", "window", `${event.message}${location}`);
  });
  target.addEventListener("unhandledrejection", (event) => {
    reportLog("error", "promise", describeError(event.reason));
  });
}
