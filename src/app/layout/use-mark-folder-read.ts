import { useMutation, useQueryClient } from "@tanstack/react-query";
import { markAllRead, messageKeys } from "@/shared/api";

export function useMarkFolderRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: markAllRead,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
