import { useMutation } from "@tanstack/react-query";
import { checkLinks } from "@/shared/api";

/** A check is a one-off request on demand; its result is not cached. */
export function useCheckLinks() {
  return useMutation({ mutationFn: checkLinks });
}
