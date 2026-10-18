import { describe, expect, it } from "vitest";
import { fieldError } from "./field-error";

describe("fieldError", () => {
  it("reads strings and issue objects and ignores the rest", () => {
    expect(fieldError([undefined, { message: "Enter a name" }])).toBe("Enter a name");
    expect(fieldError(["Too long"])).toBe("Too long");
    expect(fieldError([{ code: "x" }, null])).toBeUndefined();
  });
});
