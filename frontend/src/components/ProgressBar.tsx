import type { CSSProperties } from "react";

import styles from "./Bars.module.css";

interface ProgressBarProps {
  label: string;
  value: number;
  max: number;
  /** The value in words, such as "3 / 12". */
  valueText?: string | undefined;
}

/** The 4 px bar of the import passwords (inventario §4.26). */
export function ProgressBar({ label, value, max, valueText }: ProgressBarProps) {
  const style: CSSProperties & { "--fill": number } = { "--fill": max > 0 ? value / max : 0 };
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={value}
      aria-valuetext={valueText}
      className={styles.progress}
      style={style}
    >
      <span className={styles.progressFill} />
    </div>
  );
}
