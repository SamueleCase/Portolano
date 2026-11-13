import { useId, useState } from "react";

import { Button } from "../../components/Button";
import { GlassPanel } from "../../components/GlassPanel";
import { KeyValueGrid } from "../../components/KeyValueGrid";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { SessionHeader } from "./SessionHeader";
import type { SessionViewProps } from "./SessionView";
import styles from "./SessionViews.module.css";
import { useNow } from "./sessionServices";
import { formatDuration, isOver } from "./sessionState";

/**
 * The tab of FreeRDP in its own window (inventario §5.4 "RDP", ambiguity 12):
 * the state of its process and a duration that runs until it ends; on failure
 * "Mostra log" shows the end of its log. No "Porta in primo piano" (SPEC-UI
 * §4.5). The space below is the one the integrated RDP panel will take.
 */
export function RdpSessionView({ session, summary, route }: SessionViewProps) {
  const now = useNow(!isOver(session.state));
  const { rdp } = texts.sessions;
  const duration = formatDuration((session.endedAt ?? now) - session.startedAt);
  return (
    <div className={styles.view}>
      <SessionHeader session={session} summary={summary} route={route} />
      <div className={styles.centered}>
        {session.state.state === "failed" ? (
          <FailureCard reason={session.state.reason} />
        ) : (
          <GlassPanel className={styles.card}>
            <h3 className={styles.cardTitle}>{rdp.title}</h3>
            <p className={styles.cardText}>{rdp.text}</p>
            <KeyValueGrid
              width="session"
              rows={[
                { key: "client", label: rdp.client, value: rdp.freerdp },
                {
                  key: "duration",
                  label: rdp.duration,
                  value: <span className={styles.duration}>{duration}</span>,
                },
              ]}
            />
          </GlassPanel>
        )}
      </div>
    </div>
  );
}

/**
 * A failed FreeRDP, with its log behind "Mostra log". The log stays in the
 * page, hidden, so that the button always controls an element; it is
 * untrusted text.
 */
function FailureCard({ reason }: { reason: string }) {
  const [shown, setShown] = useState(false);
  const logId = useId();
  const { actions, failure, rdp } = texts.sessions;
  return (
    <GlassPanel className={styles.failure}>
      <h3 className={styles.failureTitle}>{failure.title}</h3>
      <div>
        <Button
          variant="small"
          aria-expanded={shown}
          aria-controls={logId}
          onClick={() => setShown((current) => !current)}
        >
          {shown ? actions.hideLog : actions.showLog}
        </Button>
      </div>
      <figure id={logId} hidden={!shown} aria-label={rdp.log} className={styles.logFigure}>
        <pre className={styles.log}>{visibleText(reason)}</pre>
      </figure>
    </GlassPanel>
  );
}
