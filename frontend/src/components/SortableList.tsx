import {
  type KeyboardEvent,
  type PointerEvent,
  type ReactNode,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";

import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { focusableIn } from "./focusable";
import styles from "./SortableList.module.css";
import { SortableRow } from "./SortableRow";
import { visibleText } from "./visibleText";

interface SortableListProps<T> {
  label: string;
  items: readonly T[];
  itemKey: (item: T) => string;
  /** The name of an item, untrusted: for the handle, "×" and the announcements. */
  itemName: (item: T) => string;
  renderItem: (item: T) => ReactNode;
  /** Moves the item at `from` to `to`; the parent owns the order and updates `items`. */
  onMove: (from: number, to: number) => void;
  /**
   * Removes the item at `index`: the parent updates `items`, and the focus
   * moves once the list is one item shorter.
   */
  onRemove: (index: number) => void;
  /** The add buttons under the list ("+ Dall'albero…"): the focus goes there when it empties. */
  footer?: ReactNode;
}

/**
 * Where the focus goes once the parent has applied a change, and how many
 * items the list has then: if the parent did not apply it (a removal it
 * postponed, for instance), the focus stays where it is.
 */
type PendingFocus = ({ kind: "handle"; key: string } | { kind: "footer" }) & { length: number };

interface Drag {
  index: number;
  startY: number;
  offset: number;
  /** The distance between two rows. */
  pitch: number;
  over: number;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/** How far the row at `index` moves while `drag` is on. */
function shiftOf(index: number, drag: Drag): number {
  if (index === drag.index) return drag.offset;
  if (drag.index < index && index <= drag.over) return -drag.pitch;
  if (drag.over <= index && index < drag.index) return drag.pitch;
  return 0;
}

/**
 * The ordered list of jump hosts (inventario §4.20, §3.4): a row is dragged
 * by its handle, the others slide to make room over a dashed placeholder;
 * from the keyboard, Alt+↑/↓ on the handle moves it, and the new position
 * is announced.
 */
export function SortableList<T>({
  label,
  items,
  itemKey,
  itemName,
  renderItem,
  onMove,
  onRemove,
  footer,
}: SortableListProps<T>) {
  const instructionsId = useId();
  const handles = useRef(new Map<string, HTMLButtonElement>());
  const [drag, setDrag] = useState<Drag>();
  const [announcement, setAnnouncement] = useState("");
  const footerBox = useRef<HTMLDivElement>(null);
  const pendingFocus = useRef<PendingFocus | undefined>(undefined);

  useEffect(() => {
    const pending = pendingFocus.current;
    pendingFocus.current = undefined;
    if (!pending || pending.length !== items.length) return;
    if (pending.kind === "handle") handles.current.get(pending.key)?.focus();
    if (pending.kind === "footer" && footerBox.current) focusableIn(footerBox.current)[0]?.focus();
  }, [items]);

  // After "×" the focus stays in the list: on the next row, else the
  // previous one, else the add buttons.
  const remove = (index: number) => {
    const neighbour = items[index + 1] ?? items[index - 1];
    const length = items.length - 1;
    pendingFocus.current = neighbour
      ? { kind: "handle", key: itemKey(neighbour), length }
      : { kind: "footer", length };
    onRemove(index);
  };

  const move = (from: number, to: number) => {
    const item = items[from];
    if (!item || from === to) return;
    onMove(from, to);
    pendingFocus.current = { kind: "handle", key: itemKey(item), length: items.length };
    setAnnouncement(
      interpolate(texts.components.sortable.moved, {
        name: visibleText(itemName(item)),
        position: to + 1,
        total: items.length,
      }),
    );
  };

  const keyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    if (!event.altKey) return;
    const step = { ArrowUp: -1, ArrowDown: 1 }[event.key];
    if (step === undefined) return;
    event.preventDefault();
    move(index, clamp(index + step, 0, items.length - 1));
  };

  const pointerDown = (event: PointerEvent<HTMLButtonElement>, index: number) => {
    const row = event.currentTarget.closest("li");
    const next = row?.nextElementSibling ?? row?.previousElementSibling;
    if (!row) return;
    const pitch = next
      ? Math.abs(next.getBoundingClientRect().top - row.getBoundingClientRect().top)
      : row.getBoundingClientRect().height;
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({ index, startY: event.clientY, offset: 0, pitch, over: index });
  };

  const pointerMove = (event: PointerEvent) => {
    if (!drag) return;
    const offset = event.clientY - drag.startY;
    const over = clamp(drag.index + Math.round(offset / drag.pitch), 0, items.length - 1);
    setDrag({ ...drag, offset, over });
  };

  const pointerUp = () => {
    if (!drag) return;
    setDrag(undefined);
    move(drag.index, drag.over);
  };

  return (
    <div className={styles.list}>
      <span id={instructionsId} className={styles.instructions}>
        {texts.components.sortable.instructions}
      </span>
      <ol
        aria-label={label}
        className={styles.rows}
        onPointerMove={pointerMove}
        onPointerUp={pointerUp}
        onPointerCancel={() => setDrag(undefined)}
      >
        {drag && (
          <li
            className={styles.placeholder}
            style={{ top: drag.over * drag.pitch }}
            aria-hidden="true"
          />
        )}
        {items.map((item, index) => {
          const key = itemKey(item);
          return (
            <SortableRow
              key={key}
              position={index + 1}
              name={visibleText(itemName(item))}
              lifted={drag?.index === index}
              offset={drag ? shiftOf(index, drag) : 0}
              instructionsId={instructionsId}
              handleRef={(element) => {
                if (element) handles.current.set(key, element);
                else handles.current.delete(key);
              }}
              onRemove={() => remove(index)}
              onHandleKeyDown={(event) => keyDown(event, index)}
              onHandlePointerDown={(event) => pointerDown(event, index)}
            >
              {renderItem(item)}
            </SortableRow>
          );
        })}
      </ol>
      {footer && (
        <div ref={footerBox} className={styles.footer}>
          {footer}
        </div>
      )}
      <span className={styles.instructions} aria-live="polite">
        {announcement}
      </span>
    </div>
  );
}
