export type MessageLink = {
  url: string;
  /** Visible text of the link in the HTML part, if it differs from the URL. */
  text: string | null;
};

const ALLOWED_PROTOCOLS = new Set(["http:", "https:", "mailto:"]);
const TEXT_URL_PATTERN = /\bhttps?:\/\/[^\s<>"'()[\]]+/g;
const TRAILING_PUNCTUATION = /[.,;:!?]+$/;

function normalize(url: string): string | null {
  try {
    const parsed = new URL(url);
    return ALLOWED_PROTOCOLS.has(parsed.protocol) ? parsed.href : null;
  } catch {
    // Relative or malformed links cannot be opened outside the mail anyway.
    return null;
  }
}

/**
 * Collects the links of a message, HTML anchors first, then URLs in the text part.
 * `DOMParser` builds an inert document: no scripts run and nothing is loaded.
 */
export function extractLinks(html: string | null, text: string | null): MessageLink[] {
  const links = new Map<string, MessageLink>();
  if (html !== null) {
    const document = new DOMParser().parseFromString(html, "text/html");
    for (const anchor of document.querySelectorAll("a[href]")) {
      const url = normalize(anchor.getAttribute("href") ?? "");
      if (url === null || links.has(url)) {
        continue;
      }
      const label = anchor.textContent?.trim() ?? "";
      links.set(url, { url, text: label === "" || label === url ? null : label });
    }
  }
  for (const match of text?.matchAll(TEXT_URL_PATTERN) ?? []) {
    const url = normalize(match[0].replace(TRAILING_PUNCTUATION, ""));
    if (url !== null && !links.has(url)) {
      links.set(url, { url, text: null });
    }
  }
  return [...links.values()];
}
