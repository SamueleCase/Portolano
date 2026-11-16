import type { ItemInstance, TreeInstance } from "@headless-tree/core";
import { type KeyboardEvent, useRef } from "react";

import { isConnection } from "../../components/nodeKind";
import type { TreeIndex } from "./treeModel";
import { TYPEAHEAD_RESET_MS, typeaheadMatch } from "./typeahead";

/** What the keys of the tree do with a node. */
export interface TreeKeyActions {
  /** Enter: the detail. */
  select: (id: string) => void;
  /** ⌘↩: a connection. */
  connect: (id: string) => void;
  /** Maiusc+F10 and the menu key: the context menu under the row. */
  openMenu: (id: string, position: { x: number; y: number }) => void;
  /** Alt+↑ / Alt+↓: one place among its siblings. */
  moveBy: (id: string, step: -1 | 1) => void;
}

/** Moves the focus of the tree to `item`, the state and the row at once. */
export function focusRow(item: ItemInstance<string> | undefined): void {
  if (!item) return;
  item.setFocused();
  // At once, not with `updateDomFocus` (a timer): a late focus of an older
  // row would take the focus of the tree back to it (the `onFocus` of a row).
  item.getElement()?.focus();
}

/**
 * The keys of the tree (SPEC-UI §5.5, WAI-ARIA tree pattern): the arrows,
 * Home and End move, → and ← open, close and go to the parent, Enter shows
 * the detail, ⌘↩ connects, Alt+↑/↓ move the node, letters jump to a name,
 * Maiusc+F10 and the menu key open the context menu. Handled here rather than
 * by the hotkeys of Headless Tree, which track the pressed keys and lose the
 * release of a key held with ⌘ (macOS sends no `keyup` for it).
 */
export function useTreeKeyboard(
  tree: TreeInstance<string>,
  index: TreeIndex,
  actions: TreeKeyActions,
  root: string,
): (event: KeyboardEvent<HTMLDivElement>) => void {
  const typed = useRef({ text: "", at: 0 });

  return (event) => {
    // The rows as the tree has them now: a key right after another (→ then ↓)
    // comes before React has drawn the rows the first one opened.
    const items = tree.getItems();
    const focused = tree.getFocusedItem();
    const id = focused.getId();
    const position = items.findIndex((item) => item.getId() === id);
    const command = event.metaKey || event.ctrlKey;

    const openMenu = () => {
      const box = focused.getElement()?.getBoundingClientRect();
      actions.openMenu(id, { x: box?.left ?? 0, y: box?.bottom ?? 0 });
    };
    const typeahead = (letter: string) => {
      const now = event.timeStamp;
      const text =
        now - typed.current.at > TYPEAHEAD_RESET_MS ? letter : typed.current.text + letter;
      typed.current = { text, at: now };
      const names = items.map((item) => item.getItemName());
      const match = typeaheadMatch(names, Math.max(position, 0), text);
      focusRow(match === undefined ? undefined : items[match]);
    };

    const handled = (() => {
      if (event.key === "Enter") {
        const entry = index.entries.get(id);
        if (!command) actions.select(id);
        else if (entry && isConnection(entry.kind)) actions.connect(id);
        return true;
      }
      if (command) return false;
      if (event.altKey) {
        if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return false;
        actions.moveBy(id, event.key === "ArrowUp" ? -1 : 1);
        return true;
      }
      switch (event.key) {
        case "ArrowDown":
          focusRow(items[position + 1]);
          return true;
        case "ArrowUp":
          focusRow(items[Math.max(0, position - 1)]);
          return true;
        case "Home":
          focusRow(items[0]);
          return true;
        case "End":
          focusRow(items.at(-1));
          return true;
        case "ArrowRight": {
          if (!focused.isFolder()) return true;
          if (!focused.isExpanded()) {
            focused.expand();
            return true;
          }
          // Into its first child, if it has one shown.
          const next = items[position + 1];
          if (next?.getParent()?.getId() === id) focusRow(next);
          return true;
        }
        case "ArrowLeft":
          if (focused.isFolder() && focused.isExpanded()) focused.collapse();
          else if (focused.getParent()?.getId() !== root) focusRow(focused.getParent());
          return true;
        case "ContextMenu":
          openMenu();
          return true;
        case "F10":
          if (!event.shiftKey) return false;
          openMenu();
          return true;
        default:
          if (event.key.length !== 1 || event.key === " ") return false;
          typeahead(event.key);
          return true;
      }
    })();
    if (handled) {
      // ⌘↩ of the tree is not also the ⌘↩ of the window.
      event.preventDefault();
      event.stopPropagation();
    }
  };
}
