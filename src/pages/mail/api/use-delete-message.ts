import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteMessagePermanently, messageKeys, projectKeys } from "@/shared/api";

export function useDeleteMessage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteMessagePermanently,
    onSuccess: (_result, id) => {
      queryClient.removeQueries({ queryKey: messageKeys.detail(id) });
      queryClient.removeQueries({ queryKey: messageKeys.raw(id) });
      return Promise.all([
        queryClient.invalidateQueries({ queryKey: messageKeys.lists() }),
        queryClient.invalidateQueries({ queryKey: messageKeys.folders() }),
        queryClient.invalidateQueries({ queryKey: projectKeys.all() }),
      ]);
    },
  });
}
