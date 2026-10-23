import { type SyntheticEvent, useState } from "react";
import { fitScale, MOBILE_WIDTH, type PreviewDevice } from "../model/preview-device";

type MailHtmlPreviewProps = {
  html: string;
  device: PreviewDevice;
};

/**
 * Mail HTML is untrusted. The sandbox blocks scripts, forms, plugins and popups;
 * `allow-same-origin` only lets this page measure the content (without
 * `allow-scripts` the mail itself still cannot run anything).
 *
 * On a phone width a mail wider than the screen is shrunk to fit, as iOS Mail does,
 * and the preview says so: the mail is not responsive.
 */
export function MailHtmlPreview({ html, device }: MailHtmlPreviewProps) {
  const [fit, setFit] = useState<{ scale: number; contentWidth: number } | null>(null);

  const handleLoad = (event: SyntheticEvent<HTMLIFrameElement>) => {
    const frame = event.currentTarget;
    const root = frame.contentDocument?.documentElement;
    if (device !== "mobile" || root === undefined) {
      setFit(null);
      return;
    }
    const contentWidth = root.scrollWidth;
    const scale = fitScale(contentWidth, frame.clientWidth);
    if (scale < 1) {
      root.style.zoom = String(scale);
    }
    setFit({ scale, contentWidth });
  };

  const shrunk = device === "mobile" && fit !== null && fit.scale < 1 ? fit : null;
  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-2">
      {shrunk === null ? null : (
        <p role="status" className="text-xs text-muted-foreground">
          Shrunk to {Math.round(shrunk.scale * 100)} %, as iOS Mail does: the mail is{" "}
          {shrunk.contentWidth} px wide and does not adapt to a phone.
        </p>
      )}
      <iframe
        // A new frame per width, so the content is measured again.
        key={device}
        title="HTML preview"
        sandbox="allow-same-origin"
        referrerPolicy="no-referrer"
        srcDoc={html}
        onLoad={handleLoad}
        style={device === "mobile" ? { width: MOBILE_WIDTH } : undefined}
        // Mail content stays on a white page, as mail clients render it.
        className={`min-h-80 flex-1 rounded-lg border bg-white ${device === "mobile" ? "" : "w-full"}`}
      />
    </div>
  );
}
