import { describe, expect, it } from "vitest";
import {
  clickSelection,
  EMPTY_SELECTION,
  isToggleClick,
  keepVisible,
  rangeBetween,
  selectOne,
} from "./selection";

const ORDER = [5, 4, 3, 2, 1];
const ids = (selection: { ids: ReadonlySet<number> }) => [...selection.ids].sort();
const plain = { toggle: false, range: false };
const toggle = { toggle: true, range: false };
const range = { toggle: false, range: true };

describe("selection", () => {
  it("selects one message with a plain click", () => {
    expect(ids(clickSelection(selectOne(5), 3, ORDER, plain))).toEqual([3]);
  });

  it("toggles single messages with the modifier", () => {
    let selection = clickSelection(selectOne(5), 3, ORDER, toggle);
    expect(ids(selection)).toEqual([3, 5]);
    selection = clickSelection(selection, 5, ORDER, toggle);
    expect(ids(selection)).toEqual([3]);
  });

  it("selects a range from the anchor with Shift, in either direction", () => {
    expect(ids(clickSelection(selectOne(4), 2, ORDER, range))).toEqual([2, 3, 4]);
    expect(ids(clickSelection(selectOne(2), 5, ORDER, range))).toEqual([2, 3, 4, 5]);
    // Shift plus the modifier adds the range to what is selected.
    const both = clickSelection(clickSelection(selectOne(5), 2, ORDER, toggle), 1, ORDER, {
      toggle: true,
      range: true,
    });
    expect(ids(both)).toEqual([1, 2, 5]);
    // Without an anchor Shift acts like a plain click.
    expect(ids(clickSelection(EMPTY_SELECTION, 3, ORDER, range))).toEqual([3]);
  });

  it("handles ids outside the list and drops messages that left it", () => {
    expect(rangeBetween(ORDER, 9, 3)).toEqual([3]);
    const selection = { ids: new Set([5, 4, 3]), anchor: 4 };
    expect(keepVisible(selection, [5, 3])).toEqual({ ids: new Set([5, 3]), anchor: null });
    expect(keepVisible(selection, ORDER)).toBe(selection);
  });

  it("uses ⌘ on macOS and Ctrl elsewhere", () => {
    expect(isToggleClick({ metaKey: true, ctrlKey: false }, true)).toBe(true);
    expect(isToggleClick({ metaKey: false, ctrlKey: true }, true)).toBe(false);
    expect(isToggleClick({ metaKey: false, ctrlKey: true }, false)).toBe(true);
  });
});
