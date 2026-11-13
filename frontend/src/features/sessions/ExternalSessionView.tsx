import { Card } from "../../components/Card";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { SessionHeader } from "./SessionHeader";
import type { SessionViewProps } from "./SessionView";
import styles from "./SessionViews.module.css";

/**
 * The tab of a connection handed to Windows App (SPEC-UI §4.5): no process to
 * follow, only "Aperta in Windows App" and "Chiudi"; if the application did
 * not open, the reason.
 */
export function ExternalSessionView({ session, summary }: SessionViewProps) {
  const { external, failure } = texts.sessions;
  return (
    <div className={styles.view}>
      <SessionHeader session={session} summary={summary} />
      <div className={styles.centered}>
        {session.state.state === "failed" ? (
          <Card large tone="error" className={styles.card}>
            <h3 className={styles.failureTitle}>{failure.title}</h3>
            <p className={styles.log}>{visibleText(session.state.reason)}</p>
          </Card>
        ) : (
          <Card large className={styles.card}>
            <h3 className={styles.cardTitle}>{external.title}</h3>
            <p className={styles.cardText}>{external.text}</p>
          </Card>
        )}
      </div>
    </div>
  );
}
