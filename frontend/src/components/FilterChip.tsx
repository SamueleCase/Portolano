import type { ReactNode } from "react";

import { classNames } from "./classNames";
import styles from "./FilterChip.module.css";

interface FilterChipProps {
  active: boolean;
  onToggle: () => void;
  /** `report`: the smaller filters of the import report (inventario §4.7). */
  variant?: "tree" | "report";
  /** The accessible name, when the visible label is a symbol such as "★". */
  label?: string;
  children: ReactNode;
}

/** A filter that can be on or off, announced with `aria-pressed`. */
export function FilterChip({
  active,
  onToggle,
  variant = "tree",
  label,
  children,
}: FilterChipProps) {
  return (
    <button
      type="button"
      aria-pressed={active}
      aria-label={label}
      className={classNames(styles.chip, variant === "report" && styles.report)}
      onClick={onToggle}
    >
      {children}
    </button>
  );
}
