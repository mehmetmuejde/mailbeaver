import { invoke, isTauri } from "@tauri-apps/api/core";
import { z } from "zod";

const serverStatusSchema = z.object({
  apiBaseUrl: z.string().nullable(),
  smtpAddress: z.string().nullable(),
  error: z.string().nullable(),
});
export type ServerStatus = z.infer<typeof serverStatusSchema>;

/** Where the embedded servers listen, or why they did not start. */
export async function fetchServerStatus(): Promise<ServerStatus> {
  if (!isTauri()) {
    // Served by the headless server: the API is on the same origin.
    return { apiBaseUrl: "", smtpAddress: null, error: null };
  }
  return serverStatusSchema.parse(await invoke("server_status"));
}
