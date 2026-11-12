import {
  type KeyboardEvent,
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

import { classNames } from "./classNames";
import styles from "./ContextMenu.module.css";
import { visibleText } from "./visibleText";

/** An entry of a context menu. */
export interface MenuItem {
  id: string;
  label: string;
  /** Shown in mono on the right, such as "⌘E". */
  shortcut?: string;
  /** "Elimina…" in red. */
  destructive?: boolean;
  disabled?: boolean;
}

/** An entry of a context menu, or a separator. */
export type MenuEntry = MenuItem | { separator: true };

interface ContextMenuProps {
  /** The accessible name, such as the name of the node. */
  label: string;
  entries: readonly MenuEntry[];
  /** Where the right click happened, in viewport pixels. */
  position: { x: number; y: number };
  onSelect: (id: string) => void;
  onClose: () => void;
}

function isItem(entry: MenuEntry): entry is MenuItem {
  return !("separator" in entry);
}

/**
 * The context menu of inventario §4.15, with the menu pattern of WAI-ARIA:
 * the arrows move between entries, Enter chooses, Esc or a click outside
 * closes, and the focus goes back where it was.
 */
export function ContextMenu({ label, entries, position, onSelect, onClose }: ContextMenuProps) {
  const menu = useRef<HTMLDivElement>(null);
  const buttons = useRef(new Map<string, HTMLButtonElement>());
  const [place, setPlace] = useState({ left: position.x, top: position.y });

  const enabledButtons = () =>
    entries
      .filter(isItem)
      .filter((item) => !item.disabled)
      .map((item) => buttons.current.get(item.id))
      .filter((button) => button !== undefined);

  useLayoutEffect(() => {
    const element = menu.current;
    if (!element) return;
    // Keeps the menu inside the window near its edges.
    const { width, height } = element.getBoundingClientRect();
    setPlace({
      left: Math.max(0, Math.min(position.x, window.innerWidth - width)),
      top: Math.max(0, Math.min(position.y, window.innerHeight - height)),
    });
  }, [position.x, position.y]);

  const closeOnOutside = useEffectEvent((event: MouseEvent) => {
    if (!(event.target instanceof Node) || !menu.current?.contains(event.target)) onClose();
  });
  const focusFirst = useEffectEvent(() => enabledButtons()[0]?.focus());

  useEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    focusFirst();
    document.addEventListener("mousedown", closeOnOutside);
    return () => {
      document.removeEventListener("mousedown", closeOnOutside);
      previous?.focus();
    };
  }, []);

  const keyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" || event.key === "Tab") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    const enabled = enabledButtons();
    const current = enabled.findIndex((button) => button === document.activeElement);
    const target = {
      ArrowDown: (current + 1) % enabled.length,
      ArrowUp: (current - 1 + enabled.length) % enabled.length,
      Home: 0,
      End: enabled.length - 1,
    }[event.key];
    if (target === undefined) return;
    event.preventDefault();
    enabled[target]?.focus();
  };

  return createPortal(
    <div
      ref={menu}
      role="menu"
      aria-label={visibleText(label)}
      className={styles.menu}
      style={{ left: place.left, top: place.top }}
      onKeyDown={keyDown}
    >
      {entries.map((entry, index) =>
        isItem(entry) ? (
          <button
            key={entry.id}
            ref={(element) => {
              if (element) buttons.current.set(entry.id, element);
              else buttons.current.delete(entry.id);
            }}
            type="button"
            role="menuitem"
            tabIndex={-1}
            disabled={entry.disabled}
            className={classNames(styles.item, entry.destructive && styles.destructive)}
            onClick={() => {
              onSelect(entry.id);
              onClose();
            }}
          >
            <span>{visibleText(entry.label)}</span>
            {/* The symbols of the shortcut read poorly: the name is the label. */}
            {entry.shortcut && (
              <span className={styles.shortcut} aria-hidden="true">
                {entry.shortcut}
              </span>
            )}
          </button>
        ) : (
          <div key={`separator-${index}`} role="separator" className={styles.separator} />
        ),
      )}
    </div>,
    document.body,
  );
}
