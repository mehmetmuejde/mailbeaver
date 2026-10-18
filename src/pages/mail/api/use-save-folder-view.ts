import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type FolderViews, saveFolderView, settingsKeys } from "@/shared/api";

/**
 * Saves a folder's view. The cache changes at once, so a quick switch to another folder
 * and back already finds the new view.
 */
export function useSaveFolderView() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: saveFolderView,
    onMutate: ({ folder, view }) => {
      queryClient.setQueryData<FolderViews>(settingsKeys.folderViews(), (views) =>
        views === undefined ? views : { ...views, [folder]: view },
      );
    },
  });
}
