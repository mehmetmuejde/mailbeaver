import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import { TooltipRoot } from "@/shared/ui/tooltip";

/** Renders with a fresh QueryClient per test; no retries so errors show at once. */
export function renderWithQuery(ui: ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <TooltipRoot>{ui}</TooltipRoot>
    </QueryClientProvider>,
  );
}
