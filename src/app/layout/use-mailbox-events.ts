import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { captureKeys, mailboxEventsUrl, messageKeys, settingsKeys } from "@/shared/api";

/**
 * Subscribes to the server's change stream and refreshes mail and settings queries.
 * Also refreshes after every (re)connect, because changes may have been missed.
 */
export function useMailboxEvents(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    const source = new EventSource(mailboxEventsUrl());
    const refresh = () => {
      void queryClient.invalidateQueries({ queryKey: messageKeys.all });
      void queryClient.invalidateQueries({ queryKey: settingsKeys.settings() });
      void queryClient.invalidateQueries({ queryKey: captureKeys.all });
    };
    source.addEventListener("change", refresh);
    source.addEventListener("open", refresh);
    return () => {
      source.removeEventListener("change", refresh);
      source.removeEventListener("open", refresh);
      source.close();
    };
  }, [queryClient]);
}
