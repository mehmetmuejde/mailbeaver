import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { onOpenMessageRequest } from "@/shared/lib/platform";

/** Shows a message picked in the tray menu of the desktop app. */
export function useOpenMessageRequests(): void {
  const navigate = useNavigate();

  useEffect(() => {
    let stop: (() => void) | undefined;
    let unmounted = false;
    onOpenMessageRequest((id) => {
      void navigate({ to: "/inbox", search: { message: id } });
    }).then(
      (unlisten) => {
        if (unmounted) {
          unlisten();
        } else {
          stop = unlisten;
        }
      },
      // Without the listener the tray still opens the window, just not the message.
      () => undefined,
    );
    return () => {
      unmounted = true;
      stop?.();
    };
  }, [navigate]);
}
