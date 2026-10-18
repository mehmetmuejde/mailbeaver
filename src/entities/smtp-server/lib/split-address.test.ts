import { describe, expect, it } from "vitest";
import { splitAddress } from "./split-address";

describe("splitAddress", () => {
  it("splits IPv4, IPv6 and port-less addresses", () => {
    expect(splitAddress("127.0.0.1:1026")).toEqual({ host: "127.0.0.1", port: "1026" });
    expect(splitAddress("[::1]:1026")).toEqual({ host: "::1", port: "1026" });
    expect(splitAddress("mailbeaver")).toEqual({ host: "mailbeaver", port: "" });
  });
});
