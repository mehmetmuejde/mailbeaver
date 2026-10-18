import { useMutation, useQueryClient } from "@tanstack/react-query";
import { messageKeys, settingsKeys, updateSettings } from "@/shared/api";

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: updateSettings,
    onSuccess: (settings) => {
      queryClient.setQueryData(settingsKeys.settings(), settings);
      // Turning on the update check makes the server ask GitHub at once.
      void queryClient.invalidateQueries({ queryKey: settingsKeys.update() });
      // A shorter retention empties expired messages from Trash right away.
      return queryClient.invalidateQueries({ queryKey: messageKeys.all });
    },
  });
}
