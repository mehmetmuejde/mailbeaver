import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteAllMessages, messageKeys } from "@/shared/api";

export function useDeleteAllMessages() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteAllMessages,
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: messageKeys.all, type: "inactive" });
      return queryClient.invalidateQueries({ queryKey: messageKeys.all });
    },
  });
}
