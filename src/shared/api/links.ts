import { z } from "zod";
import { sendJson } from "./client";

const linkStatusSchema = z.object({
  url: z.string(),
  /** HTTP status of the final response; `null` when none arrived. */
  status: z.number().int().nullable(),
  ok: z.boolean(),
  error: z.string().nullable(),
});
export type LinkStatus = z.infer<typeof linkStatusSchema>;

/** Asks the server to request each URL and report its HTTP status. */
export async function checkLinks(urls: readonly string[]): Promise<LinkStatus[]> {
  const response = await sendJson("/api/links/check", "POST", { urls });
  return z.object({ results: z.array(linkStatusSchema) }).parse(await response.json()).results;
}
