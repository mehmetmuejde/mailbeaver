import { useMutation, useQueryClient } from "@tanstack/react-query";
import { markMessageUnread, messageKeys } from "@/shared/api";

export function useMarkMessageUnread() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: markMessageUnread,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
