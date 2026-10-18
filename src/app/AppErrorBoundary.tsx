import { RotateCw } from "lucide-react";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportLog } from "@/shared/api";
import { Button } from "@/shared/ui/button";

type AppErrorBoundaryProps = {
  children: ReactNode;
};

type AppErrorBoundaryState = {
  error: Error | null;
};

/** Last line of defence: a render error shows a message instead of a blank window. */
export class AppErrorBoundary extends Component<AppErrorBoundaryProps, AppErrorBoundaryState> {
  state: AppErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): AppErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    reportLog("error", "render", `${error.message}${info.componentStack ?? ""}`);
  }

  render() {
    if (this.state.error !== null) {
      return (
        <main className="mx-auto mt-[10vh] grid max-w-2xl justify-items-start gap-3 p-4">
          <h1 className="text-lg font-semibold">Something went wrong</h1>
          <p role="alert">
            <code className="font-mono">{this.state.error.message}</code>
          </p>
          <Button icon={<RotateCw />} onClick={() => window.location.reload()}>
            Reload
          </Button>
        </main>
      );
    }
    return this.props.children;
  }
}
