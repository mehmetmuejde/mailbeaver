import { describe, expect, it } from "vitest";
import { extractLinks } from "./extract-links";

describe("extractLinks", () => {
  it("collects anchors with their text, then URLs from the text part, without duplicates", () => {
    const html = `
      <a href="https://shop.example/reset?token=abc">Reset password</a>
      <a href="https://shop.example/">https://shop.example/</a>
      <a href="mailto:help@shop.example">Help</a>
      <a href="javascript:alert(1)">Bad</a>
      <a href="/relative">Relative</a>`;
    const text = "Reset: https://shop.example/reset?token=abc. Docs (https://docs.example/start).";

    expect(extractLinks(html, text)).toEqual([
      { url: "https://shop.example/reset?token=abc", text: "Reset password" },
      { url: "https://shop.example/", text: null },
      { url: "mailto:help@shop.example", text: "Help" },
      { url: "https://docs.example/start", text: null },
    ]);
  });

  it("returns nothing without bodies", () => {
    expect(extractLinks(null, null)).toEqual([]);
  });
});
