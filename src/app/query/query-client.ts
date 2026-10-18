import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import { isClientError } from "@/shared/api";
import { QUERY_MAX_RETRIES, QUERY_STALE_TIME_MS } from "@/shared/config";
import { reportRequestError } from "../error-reporting";

export function createQueryClient(): QueryClient {
  return new QueryClient({
    // Failed requests end up in Settings → Logs, after the last retry.
    queryCache: new QueryCache({
      onError: (error, query) => reportRequestError("query", error, JSON.stringify(query.queryKey)),
    }),
    mutationCache: new MutationCache({
      onError: (error) => reportRequestError("mutation", error),
    }),
    defaultOptions: {
      queries: {
        staleTime: QUERY_STALE_TIME_MS,
        retry: (failureCount, error) => !isClientError(error) && failureCount < QUERY_MAX_RETRIES,
      },
    },
  });
}
