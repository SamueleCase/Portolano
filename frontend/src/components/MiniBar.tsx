import type { CSSProperties } from "react";

import styles from "./Bars.module.css";
import { classNames } from "./classNames";
import type { NodeKind } from "./nodeKind";

interface MiniBarProps {
  label: string;
  value: number;
  /** The largest value of the group: the bars are relative to it. */
  max: number;
  kind: NodeKind;
}

/** A bar of "Per tipo" in the import report (inventario §4.26). */
export function MiniBar({ label, value, max, kind }: MiniBarProps) {
  const style: CSSProperties & { "--fill": number } = { "--fill": max > 0 ? value / max : 0 };
  return (
    <div className={styles.mini}>
      <span className={styles.miniLabel}>{label}</span>
      <span className={styles.miniTrack} aria-hidden="true">
        <span className={classNames(styles.miniFill, styles[kind])} style={style} />
      </span>
      <span className={styles.miniValue}>{value}</span>
    </div>
  );
}
