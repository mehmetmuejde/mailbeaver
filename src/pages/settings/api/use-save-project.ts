import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createProject, messageKeys, type ProjectInput, updateProject } from "@/shared/api";

/** Creates a project, or updates it when an id is given. */
export function useSaveProject(id: number | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ProjectInput) =>
      id === null ? createProject(input) : updateProject({ id, input }),
    // New rules re-assign existing mail, so every list and count may change.
    onSuccess: () => queryClient.invalidateQueries({ queryKey: messageKeys.all }),
  });
}
