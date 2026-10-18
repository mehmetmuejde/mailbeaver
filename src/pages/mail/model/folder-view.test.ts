import { describe, expect, it } from "vitest";
import { hasViewParams, searchToView, viewToSearch } from "./folder-view";

describe("folder views", () => {
  it("turns a saved view into URL parameters and back", () => {
    const view = { grouping: "sender", project: "7", period: "7d", pageSize: 25 } as const;
    const search = viewToSearch(view);
    expect(search).toEqual({ view: "sender", project: 7, period: "7d", size: 25 });
    expect(searchToView(search)).toEqual(view);
  });

  it("leaves defaults out of the URL", () => {
    const defaults = { grouping: "all", project: null, period: null, pageSize: 50 } as const;
    expect(viewToSearch(defaults)).toEqual({
      view: undefined,
      project: undefined,
      period: undefined,
      size: undefined,
    });
    expect(viewToSearch({ ...defaults, project: "none" }).project).toBe("none");
    expect(hasViewParams({ q: "x", message: 3 })).toBe(false);
    expect(hasViewParams({ period: "today" })).toBe(true);
  });
});
