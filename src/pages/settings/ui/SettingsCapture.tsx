import { useQuery } from "@tanstack/react-query";
import { captureQueries } from "@/shared/api";
import { SwitchField } from "@/shared/ui/switch-field";
import { useSetCapture } from "../api/use-set-capture";

/** Pausing is temporary on purpose: a restart always captures again. */
export function SettingsCapture() {
  const { data } = useQuery(captureQueries.state());
  const setCapture = useSetCapture();
  return (
    <section aria-labelledby="capture-heading" className="grid gap-3">
      <h2 id="capture-heading" className="text-base font-semibold">
        Capture
      </h2>
      <SwitchField
        label="Pause capture"
        description="Refuses new mail with a temporary error (451), so senders retry later. Ends with a restart."
        checked={data?.paused ?? false}
        disabled={data === undefined || setCapture.isPending}
        onCheckedChange={(paused) => setCapture.mutate(paused)}
      />
      {setCapture.isError ? <p role="alert">Capture could not be switched.</p> : null}
    </section>
  );
}
