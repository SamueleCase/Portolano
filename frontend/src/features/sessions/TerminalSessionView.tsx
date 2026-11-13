import { Terminal } from "../../components/terminal/Terminal";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { SessionHeader } from "./SessionHeader";
import type { SessionViewProps } from "./SessionView";
import styles from "./SessionViews.module.css";
import { useSessionServices, useTerminalIo } from "./sessionServices";

/**
 * The tab of a terminal session (inventario §5.4 "SSH"): the header with the
 * single state, and the terminal, which stays after the end so that the last
 * output (the error of `ssh`, for one) can be read.
 */
export function TerminalSessionView({ session, summary, route }: SessionViewProps) {
  const { openLink, onActivity } = useSessionServices();
  const io = useTerminalIo(session.id);
  return (
    <div className={styles.view}>
      <SessionHeader session={session} summary={summary} route={route} />
      <div className={styles.terminal}>
        <Terminal
          io={io}
          onOpenLink={openLink}
          {...(onActivity && { onActivity })}
          label={interpolate(texts.sessions.terminal, { summary: visibleText(summary) })}
        />
      </div>
    </div>
  );
}
