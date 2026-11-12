import type { CSSProperties } from "react";

import styles from "./Bars.module.css";
import { classNames } from "./classNames";
import type { ImportOutcome } from "./importOutcome";

/** A segment of the bar. */
export interface BarSegment {
  outcome: ImportOutcome;
  label: string;
  count: number;
}

interface StackedBarProps {
  /** The bar in words, for assistive technologies. */
  label: string;
  segments: readonly BarSegment[];
}

/** The bar of the import report, a segment per outcome (inventario §4.26). */
export function StackedBar({ label, segments }: StackedBarProps) {
  return (
    <div role="img" aria-label={label} className={styles.stacked}>
      {segments
        .filter((segment) => segment.count > 0)
        .map((segment) => {
          const style: CSSProperties & { "--share": number } = { "--share": segment.count };
          return (
            <span
              key={segment.outcome}
              className={classNames(styles.segment, styles[segment.outcome])}
              style={style}
            />
          );
        })}
    </div>
  );
}
