import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { captureKeys, captureQueries, setCapturePaused } from "@/shared/api";

/** Whether capture is paused, and the switch for it. */
export function useCapture() {
  const queryClient = useQueryClient();
  const { data } = useQuery(captureQueries.state());
  const toggle = useMutation({
    mutationFn: setCapturePaused,
    onSuccess: (state) => queryClient.setQueryData(captureKeys.all, state),
  });
  return { paused: data?.paused ?? false, toggle };
}
