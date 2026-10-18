import { useMutation, useQueryClient } from "@tanstack/react-query";
import { captureKeys, setCapturePaused } from "@/shared/api";

export function useSetCapture() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setCapturePaused,
    onSuccess: (state) => queryClient.setQueryData(captureKeys.all, state),
  });
}
