import type { CSSProperties, KeyboardEvent, PointerEvent, ReactNode, Ref } from "react";

import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./SortableList.module.css";

interface SortableRowProps {
  /** The position shown, from 1. */
  position: number;
  /** The visible name, already made visible, for the handle and "×". */
  name: string;
  /** Lifted while dragged: shadow, accent border, slight tilt (inventario §3.4). */
  lifted?: boolean;
  /** The id of the text that explains the keyboard. */
  instructionsId: string;
  /** Where the row is drawn while another one is dragged. */
  offset?: number;
  onRemove: () => void;
  onHandleKeyDown?: (event: KeyboardEvent<HTMLButtonElement>) => void;
  onHandlePointerDown?: (event: PointerEvent<HTMLButtonElement>) => void;
  handleRef?: Ref<HTMLButtonElement>;
  children: ReactNode;
}

/** A row of the jump host list: handle, position, content, "×" (inventario §4.20). */
export function SortableRow({
  position,
  name,
  lifted = false,
  instructionsId,
  offset = 0,
  onRemove,
  onHandleKeyDown,
  onHandlePointerDown,
  handleRef,
  children,
}: SortableRowProps) {
  const style: CSSProperties & { "--row-offset": string } = { "--row-offset": `${offset}px` };
  return (
    <li className={classNames(styles.row, lifted && styles.lifted)} style={style}>
      <button
        ref={handleRef}
        type="button"
        className={styles.handle}
        aria-label={interpolate(texts.components.sortable.handle, { name })}
        aria-describedby={instructionsId}
        onKeyDown={onHandleKeyDown}
        onPointerDown={onHandlePointerDown}
      >
        ⋮⋮
      </button>
      <span className={styles.position}>{position}</span>
      <span className={styles.content}>{children}</span>
      <button
        type="button"
        className={styles.remove}
        aria-label={interpolate(texts.components.sortable.remove, { name })}
        onClick={onRemove}
      >
        ×
      </button>
    </li>
  );
}
