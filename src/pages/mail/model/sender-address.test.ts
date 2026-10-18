import { describe, expect, it } from "vitest";
import { senderAddress } from "./sender-address";

describe("senderAddress", () => {
  it("takes the address from a display name form or keeps a bare address", () => {
    expect(senderAddress("Shop <noreply@shop.example>")).toBe("noreply@shop.example");
    expect(senderAddress(" a@b.example ")).toBe("a@b.example");
  });
});
