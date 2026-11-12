import type { ReactNode } from "react";

import { classNames } from "./classNames";
import styles from "./KeyValueGrid.module.css";

/** A row of the grid: the label and its value. */
export interface KeyValueRow {
  key: string;
  label: string;
  value: ReactNode;
  /** Hosts, users, paths and custom properties. */
  mono?: boolean;
}

interface KeyValueGridProps {
  rows: readonly KeyValueRow[];
  /** The label column: 150 (detail), 140 (RDP), 110 (selector), 90 (import). */
  width?: "detail" | "session" | "selector" | "import";
}

/** Labels and values on two columns, as a description list (inventario §4.26). */
export function KeyValueGrid({ rows, width = "detail" }: KeyValueGridProps) {
  return (
    <dl className={classNames(styles.grid, styles[width])}>
      {rows.map((row) => (
        <div key={row.key} className={styles.row}>
          <dt className={styles.label}>{row.label}</dt>
          <dd className={classNames(styles.value, row.mono && styles.mono)}>{row.value}</dd>
        </div>
      ))}
    </dl>
  );
}
