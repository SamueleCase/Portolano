import styles from "./Spinner.module.css";

/** The 12 px spinner of a busy button (inventario §4.1, proposal). Decorative. */
export function Spinner() {
  return <span className={styles.spinner} aria-hidden="true" />;
}
