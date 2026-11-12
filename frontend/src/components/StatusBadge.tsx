import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./StatusBadge.module.css";

/** The states a status badge shows (inventario §4.10). */
export type BadgeStatus = "connected" | "connecting" | "error" | "warning" | "info";

/**
 * A pill with a colored border and a marker: dot when connected, hollow
 * circle while connecting, diamond on error (inventario §4.10).
 */
export function StatusBadge({ status, label }: { status: BadgeStatus; label?: string }) {
  return (
    <span className={classNames(styles.badge, styles[status])}>
      <span className={styles.marker} aria-hidden="true" />
      {label ?? texts.components.statusBadge[status]}
    </span>
  );
}
