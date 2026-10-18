import type { PreviewDevice } from "../model/preview-device";

/** Width of a typical phone viewport in CSS pixels. */
const MOBILE_WIDTH_CLASS = "w-[390px]";

type MailHtmlPreviewProps = {
  html: string;
  device: PreviewDevice;
};

/**
 * Mail HTML is untrusted: the empty `sandbox` blocks scripts, forms, plugins and
 * same-origin access, so the content can only be looked at.
 */
export function MailHtmlPreview({ html, device }: MailHtmlPreviewProps) {
  return (
    <iframe
      title="HTML preview"
      sandbox=""
      referrerPolicy="no-referrer"
      srcDoc={html}
      // Mail content stays on a white page, as mail clients render it.
      className={`min-h-80 flex-1 self-center rounded-lg border bg-white ${
        device === "mobile" ? MOBILE_WIDTH_CLASS : "w-full"
      }`}
    />
  );
}
