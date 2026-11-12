import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./SeverityMarker.module.css";

/** The levels of the validation (DEC-016.6). */
export type Severity = "error" | "warning" | "info";

interface SeverityMarkerProps {
  severity: Severity;
  /** Shows the word next to the shape: severity is never told by color alone. */
  withLabel?: boolean;
}

/** Error = diamond, warning = dot, note = hollow square (inventario §4.11). */
export function SeverityMarker({ severity, withLabel = false }: SeverityMarkerProps) {
  const word = texts.components.severity[severity];
  return (
    <span className={classNames(styles.severity, styles[severity])}>
      {withLabel ? (
        <>
          <span className={styles.marker} aria-hidden="true" />
          <span className={styles.label}>{word}</span>
        </>
      ) : (
        <span className={styles.marker} role="img" aria-label={word} />
      )}
    </span>
  );
}
