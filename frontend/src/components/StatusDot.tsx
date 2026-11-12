import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./StatusDot.module.css";

/**
 * What a dot tells (inventario §4.13): the session of a node or a tab, or the
 * kind of a tab (detail, validation).
 */
export type DotState = "idle" | "connecting" | "connected" | "failed" | "detail" | "validation";

interface StatusDotProps {
  state: DotState;
  /** `large`: 9 px in the session header; 7 px elsewhere. */
  size?: "normal" | "large";
}

/**
 * A colored dot with its meaning as text for assistive technologies: the
 * state is never only a color (inventario §7). "idle" draws nothing.
 */
export function StatusDot({ state, size = "normal" }: StatusDotProps) {
  if (state === "idle") {
    return <span className={styles.dot} aria-hidden="true" />;
  }
  return (
    <span
      role="img"
      aria-label={texts.components.statusDot[state]}
      className={classNames(styles.dot, styles[state], size === "large" && styles.large)}
    />
  );
}
