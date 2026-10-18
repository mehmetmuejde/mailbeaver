import { useMutation, useQueryClient } from "@tanstack/react-query";
import { markMessageRead, messageKeys, projectKeys } from "@/shared/api";

export function useMarkMessageRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: markMessageRead,
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: messageKeys.lists() }),
        queryClient.invalidateQueries({ queryKey: messageKeys.folders() }),
        queryClient.invalidateQueries({ queryKey: projectKeys.all() }),
      ]),
  });
}
