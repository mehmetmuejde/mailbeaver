import { useMutation, useQueryClient } from "@tanstack/react-query";
import { messageKeys, moveMessage } from "@/shared/api";

export function useMoveMessage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: moveMessage,
    // Lists, counts and the detail (its folder decides the actions) all change.
    onSuccess: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
