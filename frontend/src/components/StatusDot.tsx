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
  /** When the words next to it already tell the same. */
  decorative?: boolean;
}

/**
 * A colored dot with its meaning as text for assistive technologies: the
 * state is never only a color (inventario §7). "idle" draws nothing.
 */
export function StatusDot({ state, size = "normal", decorative = false }: StatusDotProps) {
  const className = classNames(styles.dot, styles[state], size === "large" && styles.large);
  if (state === "idle" || decorative) {
    return <span className={className} aria-hidden="true" />;
  }
  return <span role="img" aria-label={texts.components.statusDot[state]} className={className} />;
}
