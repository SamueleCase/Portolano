import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./Logo.module.css";

interface LogoProps {
  /** `toolbar`: mark 24 px and wordmark 16 px; `first-run`: 30 px and 22 px. */
  size?: "toolbar" | "first-run";
  /** Shows the "portolano" wordmark next to the mark. */
  wordmark?: boolean;
}

/**
 * The mark of the App board (ambiguity 3): a route drawing a "P" through four
 * ports, in the accent text color (inventario §1.2).
 */
export function Logo({ size = "toolbar", wordmark = true }: LogoProps) {
  return (
    <span className={classNames(styles.logo, size === "first-run" && styles.large)}>
      <svg
        viewBox="0 0 64 64"
        className={styles.mark}
        {...(wordmark
          ? { "aria-hidden": true }
          : { role: "img", "aria-label": texts.components.logo.name })}
      >
        <polyline points="20,54 20,12 40,12 48,22 40,32 20,32" />
        <circle cx="20" cy="54" r="5" />
        <circle cx="20" cy="12" r="5" />
        <circle cx="48" cy="22" r="5" />
        <circle cx="20" cy="32" r="5" />
      </svg>
      {wordmark && <span className={styles.wordmark}>{texts.components.logo.wordmark}</span>}
    </span>
  );
}
