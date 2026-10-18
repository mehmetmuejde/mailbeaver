import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteProject, messageKeys } from "@/shared/api";

export function useDeleteProject() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteProject,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
