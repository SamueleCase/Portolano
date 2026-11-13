import { Button } from "../../components/Button";
import { GlassPanel } from "../../components/GlassPanel";
import { StatusDot } from "../../components/StatusDot";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import styles from "./SessionViews.module.css";
import { useSessionAction, useSessionServices } from "./sessionServices";
import { canRetry, isRunning, sessionDot, statusLabel } from "./sessionState";
import type { Session } from "./store";

/** The size a session reopened by "Riprova" starts with; its terminal fits it at once. */
const RETRY_SIZE = { cols: 80, rows: 24 };

interface SessionHeaderProps {
  session: Session;
  /** The summary of the connection, such as `user@host:22`: untrusted text. */
  summary: string;
}

/**
 * The head of a session tab (inventario §5.4): the dot and the state of the
 * whole connection (ambiguity 11), "Riprova" when it did not succeed, and
 * "Disconnetti" while its program runs, "Chiudi" afterwards.
 */
export function SessionHeader({ session, summary }: SessionHeaderProps) {
  const { controller } = useSessionServices();
  const { run, error } = useSessionAction();
  const { actions } = texts.sessions;
  const running = isRunning(session.state);
  return (
    <GlassPanel as="header" variant="inner" aria-label={texts.sessions.header}>
      <div className={styles.headerRow}>
        <StatusDot state={sessionDot(session.state)} size="large" decorative />
        <span className={styles.summary}>{visibleText(summary)}</span>
        <span role="status" className={styles[session.state.state]}>
          {statusLabel(session.kind, session.state)}
        </span>
        <span className={styles.spacer} />
        {canRetry(session.state) && (
          <Button
            variant="primary"
            onClick={() => run(() => controller.retry(session.id, { size: RETRY_SIZE }))}
          >
            {actions.retry}
          </Button>
        )}
        {running && session.kind !== "external" ? (
          <Button onClick={() => run(() => controller.disconnect(session.id))}>
            {actions.disconnect}
          </Button>
        ) : (
          <Button onClick={() => run(() => controller.close(session.id))}>{actions.close}</Button>
        )}
      </div>
      {error !== undefined && (
        <p role="alert" className={styles.actionError}>
          {visibleText(error)}
        </p>
      )}
    </GlassPanel>
  );
}
