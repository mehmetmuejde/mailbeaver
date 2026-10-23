/** Width the HTML preview simulates. */
export type PreviewDevice = "desktop" | "mobile";

/** Width of a typical phone viewport in CSS pixels. */
export const MOBILE_WIDTH = 390;

/**
 * Phones such as iOS Mail shrink a mail that is wider than the screen until it fits;
 * 1 when the content fits already.
 */
export function fitScale(contentWidth: number, viewportWidth: number): number {
  if (viewportWidth <= 0 || contentWidth <= viewportWidth) {
    return 1;
  }
  return viewportWidth / contentWidth;
}
