import { percent } from "../i18n/format";
import styles from "./Bars.module.css";
import { classNames } from "./classNames";
import type { ImportOutcome } from "./importOutcome";

interface StatTileProps {
  outcome: ImportOutcome;
  label: string;
  count: number;
  total: number;
}

/** A tile of the import report: label, count and share (inventario §4.26). */
export function StatTile({ outcome, label, count, total }: StatTileProps) {
  return (
    <div className={styles.tile}>
      <span className={styles.tileLabel}>
        <span className={classNames(styles.tileDot, styles[outcome])} aria-hidden="true" />
        {label}
      </span>
      <span className={styles.tileValue}>{count}</span>
      <span className={styles.tilePercent}>{percent(count, total)}</span>
    </div>
  );
}
