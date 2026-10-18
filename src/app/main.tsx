import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { configureApiBaseUrl, fetchServerStatus, type ServerStatus } from "@/shared/api";
import { initTheme } from "@/shared/lib/theme";
import { App } from "./App";
import { AppErrorBoundary } from "./AppErrorBoundary";
import { installErrorReporting } from "./error-reporting";
import { createQueryClient } from "./query/query-client";
import "./styles/global.css";

// Before the first render, so the page never flashes in the wrong theme.
initTheme();
installErrorReporting();

const container = document.getElementById("root");
if (container === null) {
  throw new Error("Root element #root is missing in index.html");
}
const root = createRoot(container);
const queryClient = createQueryClient();

function render(status: ServerStatus): void {
  root.render(
    <StrictMode>
      <AppErrorBoundary>
        <QueryClientProvider client={queryClient}>
          <App status={status} />
        </QueryClientProvider>
      </AppErrorBoundary>
    </StrictMode>,
  );
}

// The API address must be known before the first query runs.
fetchServerStatus().then(
  (status) => {
    if (status.apiBaseUrl !== null) {
      configureApiBaseUrl(status.apiBaseUrl);
    }
    render(status);
  },
  (error: unknown) => {
    render({
      apiBaseUrl: null,
      smtpAddress: null,
      error: error instanceof Error ? error.message : String(error),
    });
  },
);
