import { queryOptions } from "@tanstack/react-query";
import { z } from "zod";
import { requestJson, sendJson } from "./client";

const captureSchema = z.object({
  /** While paused, the SMTP server refuses mail with a temporary error (451). */
  paused: z.boolean(),
});
type CaptureState = z.infer<typeof captureSchema>;

function fetchCapture(): Promise<CaptureState> {
  return requestJson("/api/capture", captureSchema);
}

export async function setCapturePaused(paused: boolean): Promise<CaptureState> {
  const response = await sendJson("/api/capture", "PUT", { paused });
  return captureSchema.parse(await response.json());
}

export const captureKeys = {
  all: ["capture"] as const,
};

export const captureQueries = {
  state: () => queryOptions({ queryKey: captureKeys.all, queryFn: fetchCapture }),
};
