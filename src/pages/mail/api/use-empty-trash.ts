import { useMutation, useQueryClient } from "@tanstack/react-query";
import { emptyTrash, messageKeys } from "@/shared/api";

export function useEmptyTrash() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: emptyTrash,
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: messageKeys.all, type: "inactive" });
      return queryClient.invalidateQueries({ queryKey: messageKeys.all });
    },
  });
}
