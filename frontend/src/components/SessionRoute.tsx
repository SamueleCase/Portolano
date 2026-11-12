import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./SessionRoute.module.css";
import { visibleText } from "./visibleText";
import { VisuallyHidden } from "./VisuallyHidden";

/**
 * The single state of a connection (ambiguity 11): OpenSSH with `-J` tells
 * no progress per jump, so the whole route shares one phase.
 */
export type ConnectionPhase = "connecting" | "connected" | "failed";

interface SessionRouteProps {
  /** The jumps and the destination, after "tu": untrusted text. */
  stops: readonly string[];
  phase: ConnectionPhase;
}

/**
 * The route in the session header (inventario §4.22, §3.3): all in the accent
 * while connecting, all in green once connected; on failure the lines go
 * muted and the destination turns red.
 */
export function SessionRoute({ stops, phase }: SessionRouteProps) {
  const labels = [texts.components.route.origin, ...stops.map(visibleText)];
  return (
    <div className={classNames(styles.route, styles[phase])}>
      <VisuallyHidden>{texts.components.route.phases[phase]}</VisuallyHidden>
      <ol aria-label={texts.components.route.label} className={styles.stops}>
        {labels.map((label, index) => (
          <li
            key={`${index}-${label}`}
            className={classNames(styles.stop, index === labels.length - 1 && styles.destination)}
          >
            {index > 0 && <span className={styles.line} aria-hidden="true" />}
            <span className={styles.point} aria-hidden="true" />
            <span className={styles.label}>{label}</span>
          </li>
        ))}
      </ol>
    </div>
  );
}
