import { type DragEvent, type ReactNode, useState } from "react";

import { classNames } from "./classNames";
import styles from "./DropZone.module.css";

interface DropZoneProps {
  /** "Trascina qui il documento di Royal TSX". */
  title: string;
  /** "File .rtsz, .rtsx o XML, oppure". */
  hint: string;
  /** The button that opens the native dialog (paths are chosen by Rust). */
  action: ReactNode;
  /** A file dropped: the frontend only learns that a drop happened. */
  onDrop: () => void;
}

/** The dashed area of the import (inventario §4.26, §3.4). */
export function DropZone({ title, hint, action, onDrop }: DropZoneProps) {
  const [over, setOver] = useState(false);
  const accept = (event: DragEvent) => {
    event.preventDefault();
    setOver(true);
  };
  return (
    <div
      className={classNames(styles.zone, over && styles.over)}
      onDragEnter={accept}
      onDragOver={accept}
      onDragLeave={() => setOver(false)}
      onDrop={(event) => {
        event.preventDefault();
        setOver(false);
        onDrop();
      }}
    >
      <span className={styles.title}>{title}</span>
      <span className={styles.hint}>{hint}</span>
      {action}
    </div>
  );
}
