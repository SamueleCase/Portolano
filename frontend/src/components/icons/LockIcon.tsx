import { classNames } from "../classNames";
import styles from "./Icon.module.css";
import { type IconSize, iconSizeClass } from "./iconSizes";

const CLOSED = "M5 7V5a3 3 0 0 1 6 0v2M4 7h8v6H4z";
const OPEN = "M5 7V5a3 3 0 0 1 5.8-1M4 7h8v6H4z";

/** The padlock of the secret badges and the vault (inventario §4.9). Decorative. */
export function LockIcon({ open = false, size = "badge" }: { open?: boolean; size?: IconSize }) {
  return (
    <svg
      viewBox="0 0 16 16"
      aria-hidden="true"
      className={classNames(styles.icon, iconSizeClass[size])}
    >
      <path d={open ? OPEN : CLOSED} />
    </svg>
  );
}
