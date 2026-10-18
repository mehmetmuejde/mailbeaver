import { describe, expect, it } from "vitest";
import { ApiError } from "@/shared/api";
import { createQueryClient } from "./query-client";

describe("createQueryClient", () => {
  const retry = createQueryClient().getDefaultOptions().queries?.retry;
  if (typeof retry !== "function") {
    throw new Error("retry must be a function");
  }

  it("never retries client errors", () => {
    expect(retry(0, new ApiError(404, "missing"))).toBe(false);
  });

  it("retries other failures a limited number of times", () => {
    const failure = new ApiError(500, "boom");
    expect(retry(0, failure)).toBe(true);
    expect(retry(1, failure)).toBe(true);
    expect(retry(2, failure)).toBe(false);
  });
});
