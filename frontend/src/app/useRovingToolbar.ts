import { type FocusEvent, type KeyboardEvent, useState } from "react";

/** The attribute that marks the items of a toolbar, with their position. */
const ITEM = "data-toolbar-item";

/** The items of the toolbar `container`, in order. */
function itemsOf(container: HTMLElement): HTMLElement[] {
  return [...container.querySelectorAll<HTMLElement>(`[${ITEM}]`)];
}

/**
 * The roving tabindex of a `toolbar` (WAI-ARIA toolbar pattern): Tab enters
 * and leaves the bar on one item, ← and → move among the items, Home and
 * End go to the first and the last; the last item reached is the one Tab
 * comes back to. Each item carries `data-toolbar-item={position}` and the
 * `tabIndex` of `tabIndex(position)`.
 */
export function useRovingToolbar() {
  const [active, setActive] = useState(0);
  return {
    tabIndex: (position: number) => (position === active ? 0 : -1),
    onFocus: (event: FocusEvent<HTMLElement>) => {
      const item = event.target.closest<HTMLElement>(`[${ITEM}]`);
      const position = Number(item?.getAttribute(ITEM));
      if (item && Number.isInteger(position)) setActive(position);
    },
    onKeyDown: (event: KeyboardEvent<HTMLElement>) => {
      const items = itemsOf(event.currentTarget);
      const current = items.findIndex((item) => item.contains(document.activeElement));
      if (current === -1) return;
      const target = {
        ArrowRight: (current + 1) % items.length,
        ArrowLeft: (current - 1 + items.length) % items.length,
        Home: 0,
        End: items.length - 1,
      }[event.key];
      if (target === undefined) return;
      event.preventDefault();
      items[target]?.focus();
    },
  };
}
