/**
 * Multi-selection of the message list, as mail clients do it: a plain click selects
 * one message, the platform modifier (⌘ on macOS, Ctrl elsewhere) toggles one, Shift
 * extends from the last clicked message. Pure, so the rules are tested without a DOM.
 */
export type Selection = {
  ids: ReadonlySet<number>;
  /** Where a Shift range starts: the last message clicked without Shift. */
  anchor: number | null;
};

export const EMPTY_SELECTION: Selection = { ids: new Set(), anchor: null };

export type ClickModifiers = {
  /** ⌘ on macOS, Ctrl on Windows and Linux. */
  toggle: boolean;
  range: boolean;
};

export function selectOne(id: number): Selection {
  return { ids: new Set([id]), anchor: id };
}

/** The ids between `from` and `to` in list order, both included. */
export function rangeBetween(order: readonly number[], from: number, to: number): number[] {
  const start = order.indexOf(from);
  const end = order.indexOf(to);
  if (start === -1 || end === -1) {
    return [to];
  }
  return order.slice(Math.min(start, end), Math.max(start, end) + 1);
}

export function clickSelection(
  selection: Selection,
  id: number,
  order: readonly number[],
  modifiers: ClickModifiers,
): Selection {
  if (modifiers.range && selection.anchor !== null) {
    const range = rangeBetween(order, selection.anchor, id);
    const ids = modifiers.toggle ? new Set([...selection.ids, ...range]) : new Set(range);
    return { ids, anchor: selection.anchor };
  }
  if (modifiers.toggle) {
    const ids = new Set(selection.ids);
    if (ids.has(id)) {
      ids.delete(id);
    } else {
      ids.add(id);
    }
    return { ids, anchor: id };
  }
  return selectOne(id);
}

/** Keeps only messages still in the list, e.g. after some were moved away. */
export function keepVisible(selection: Selection, order: readonly number[]): Selection {
  const visible = new Set(order);
  const ids = new Set([...selection.ids].filter((id) => visible.has(id)));
  if (ids.size === selection.ids.size) {
    return selection;
  }
  const anchor =
    selection.anchor !== null && visible.has(selection.anchor) ? selection.anchor : null;
  return { ids, anchor };
}

/** macOS uses ⌘ to toggle; there Ctrl-click opens the context menu. */
export function isToggleClick(
  event: { metaKey: boolean; ctrlKey: boolean },
  isMac: boolean,
): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}
