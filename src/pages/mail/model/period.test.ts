import { describe, expect, it } from "vitest";
import { periodStart } from "./period";

describe("periodStart", () => {
  const now = new Date(2026, 9, 17, 15, 30);

  it("starts at local midnight of the first day of the period", () => {
    expect(periodStart("today", now)).toBe(new Date(2026, 9, 17).getTime());
    expect(periodStart("7d", now)).toBe(new Date(2026, 9, 11).getTime());
    expect(periodStart("30d", now)).toBe(new Date(2026, 8, 18).getTime());
  });
});
