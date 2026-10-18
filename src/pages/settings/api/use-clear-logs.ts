import { useMutation, useQueryClient } from "@tanstack/react-query";
import { clearLogs, logKeys } from "@/shared/api";

export function useClearLogs() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: clearLogs,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: logKeys.all }),
  });
}
