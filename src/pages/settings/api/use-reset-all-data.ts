import { useMutation, useQueryClient } from "@tanstack/react-query";
import { resetAllData } from "@/shared/api";

export function useResetAllData() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: resetAllData,
    // Messages, projects and settings all changed.
    onSuccess: () => queryClient.invalidateQueries(),
  });
}
