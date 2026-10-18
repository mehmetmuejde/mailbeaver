import { useQueryClient } from "@tanstack/react-query";
import { type RouterHistory, RouterProvider } from "@tanstack/react-router";
import { useState } from "react";
import { ServerErrorPage } from "@/pages/server-error";
import type { ServerStatus } from "@/shared/api";
import { TooltipRoot } from "@/shared/ui/tooltip";
import { createAppRouter } from "./router";

type AppProps = {
  status: ServerStatus;
  /** Tests pass a memory history; the app uses the URL hash. */
  history?: RouterHistory;
};

export function App({ status, history }: AppProps) {
  if (status.error !== null || status.apiBaseUrl === null) {
    return <ServerErrorPage message={status.error ?? "The mail server is not running."} />;
  }
  return <AppRouter history={history} />;
}

type AppRouterProps = {
  history?: RouterHistory;
};

function AppRouter({ history }: AppRouterProps) {
  const queryClient = useQueryClient();
  const [router] = useState(() => createAppRouter({ queryClient, history }));
  return (
    <TooltipRoot>
      <RouterProvider router={router} />
    </TooltipRoot>
  );
}
