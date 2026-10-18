import { useMutation, useQueryClient } from "@tanstack/react-query";
import { saveServerSettings, settingsKeys } from "@/shared/api";

export function useSaveServerSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: saveServerSettings,
    onSuccess: (saved) => queryClient.setQueryData(settingsKeys.server(), saved),
  });
}
