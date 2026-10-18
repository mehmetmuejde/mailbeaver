import { Play } from "lucide-react";
import { Button } from "@/shared/ui/button";
import { useCapture } from "./use-capture";

/** Visible on every page while Mailbeaver refuses mail, so nobody wonders why. */
export function CapturePausedBanner() {
  const { paused, toggle } = useCapture();
  if (!paused) {
    return null;
  }
  return (
    <div
      role="status"
      className="flex items-center gap-3 border-b bg-amber-100 px-4 py-2 text-sm text-amber-950 dark:bg-amber-950 dark:text-amber-100"
    >
      <p className="flex-1">
        Capture is paused: new mail is refused with a temporary error, so senders retry later.
      </p>
      <Button
        size="sm"
        icon={<Play />}
        disabled={toggle.isPending}
        onClick={() => toggle.mutate(false)}
      >
        Resume capture
      </Button>
      {toggle.isError ? <p role="alert">Capture could not be resumed.</p> : null}
    </div>
  );
}
