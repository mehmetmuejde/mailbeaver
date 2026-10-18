import { describe, expect, it } from "vitest";
import { mailSearchSchema } from "./mail-search";

describe("mailSearchSchema", () => {
  it("keeps valid values", () => {
    expect(mailSearchSchema.parse({ q: "reset", view: "sender", project: 2, message: 4 })).toEqual({
      q: "reset",
      view: "sender",
      project: 2,
      message: 4,
    });
    expect(mailSearchSchema.parse({ project: "none" })).toEqual({ project: "none" });
  });

  it("drops invalid values instead of failing", () => {
    expect(
      mailSearchSchema.parse({ view: "cards", message: -1, q: "x".repeat(300), project: "shop" }),
    ).toEqual({});
  });
});
