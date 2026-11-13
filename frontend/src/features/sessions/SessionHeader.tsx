import { Button } from "../../components/Button";
import { GlassPanel } from "../../components/GlassPanel";
import { type ConnectionPhase, SessionRoute } from "../../components/SessionRoute";
import { StatusDot } from "../../components/StatusDot";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import type { SessionState } from "../../ipc/sessions";
import styles from "./SessionViews.module.css";
import { useSessionAction, useSessionServices } from "./sessionServices";
import { canRetry, sessionDot, statusLabel } from "./sessionState";
import type { Session } from "./store";

/** The size a session reopened by "Riprova" starts with; its terminal fits it at once. */
const RETRY_SIZE = { cols: 80, rows: 24 };

interface SessionHeaderProps {
  session: Session;
  /** The summary of the connection, such as `user@host:22`: untrusted text. */
  summary: string;
  /** The jumps and the destination, for the route under the header. */
  route?: readonly string[] | undefined;
}

/** The phase of the route: an ended session has none to show. */
function routePhase(state: SessionState): ConnectionPhase | undefined {
  return state.state === "ended" ? undefined : state.state;
}

/**
 * The head of a session tab (inventario §5.4): the dot and the state of the
 * whole connection (ambiguity 11), "Riprova" when it did not succeed,
 * "Disconnetti" once connected and "Chiudi" otherwise, and the route of the
 * connection below.
 */
export function SessionHeader({ session, summary, route }: SessionHeaderProps) {
  const services = useSessionServices();
  const { controller } = services;
  const { run, pending, error } = useSessionAction();
  const { actions } = texts.sessions;
  const phase = routePhase(session.state);
  // With Windows App the connection is not ours to end.
  const connected = session.state.state === "connected" && session.kind !== "external";
  const retry = async () => {
    const again = services.onRetry ? await services.onRetry(session) : {};
    if (again === null) return;
    const next = await controller.retry(session.id, RETRY_SIZE, again.login);
    services.onReplaced?.(session.id, next);
  };
  return (
    <GlassPanel variant="inner" role="group" aria-label={texts.sessions.header}>
      <div className={styles.headerRow}>
        <StatusDot state={sessionDot(session.state)} size="large" decorative />
        <span className={styles.summary}>{visibleText(summary)}</span>
        <span role="status" className={styles[session.state.state]}>
          {statusLabel(session.kind, session.state)}
        </span>
        <span className={styles.spacer} />
        {canRetry(session.state) && (
          <Button variant="primary" loading={pending} onClick={() => run(retry)}>
            {actions.retry}
          </Button>
        )}
        {connected ? (
          <Button loading={pending} onClick={() => run(() => controller.disconnect(session.id))}>
            {actions.disconnect}
          </Button>
        ) : (
          <Button loading={pending} onClick={() => run(() => controller.close(session.id))}>
            {actions.close}
          </Button>
        )}
      </div>
      {route !== undefined && phase !== undefined && (
        <div className={styles.route}>
          <SessionRoute stops={route} phase={phase} />
        </div>
      )}
      {error !== undefined && (
        <p role="alert" className={styles.actionError}>
          {visibleText(error)}
        </p>
      )}
    </GlassPanel>
  );
}
