import { describe, expect, it } from "vitest";
import { settingsSearchSchema } from "./settings-search";

describe("settingsSearchSchema", () => {
  it("keeps a known tab and drops an unknown one", () => {
    expect(settingsSearchSchema.parse({ tab: "projects" })).toEqual({ tab: "projects" });
    expect(settingsSearchSchema.parse({ tab: "secrets" })).toEqual({});
  });
});
