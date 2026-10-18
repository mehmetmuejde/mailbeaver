import type { ErrorComponentProps } from "@tanstack/react-router";
import { RotateCw } from "lucide-react";
import { Button } from "@/shared/ui/button";

export function RouteError({ error, reset }: ErrorComponentProps) {
  return (
    <div role="alert" className="mx-auto grid max-w-xl justify-items-start gap-3 p-6">
      <h1 className="text-lg font-semibold">This view could not be shown</h1>
      <p>
        <code className="font-mono">{error instanceof Error ? error.message : String(error)}</code>
      </p>
      <Button icon={<RotateCw />} onClick={reset}>
        Try again
      </Button>
    </div>
  );
}
