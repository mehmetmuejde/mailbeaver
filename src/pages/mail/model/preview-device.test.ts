import { describe, expect, it } from "vitest";
import { fitScale } from "./preview-device";

describe("fitScale", () => {
  it("shrinks only content wider than the viewport", () => {
    expect(fitScale(600, 390)).toBeCloseTo(0.65);
    expect(fitScale(390, 390)).toBe(1);
    expect(fitScale(320, 390)).toBe(1);
    expect(fitScale(600, 0)).toBe(1);
  });
});
