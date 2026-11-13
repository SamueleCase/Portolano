import { Terminal } from "../../components/terminal/Terminal";
import { SessionHeader } from "./SessionHeader";
import type { SessionViewProps } from "./SessionView";
import styles from "./SessionViews.module.css";
import { useSessionServices, useTerminalIo } from "./sessionServices";

/**
 * The tab of a terminal session (inventario §5.4 "SSH"): the header with the
 * single state, and the terminal, which stays after the end so that the last
 * output (the error of `ssh`, for one) can be read.
 */
export function TerminalSessionView({ session, summary }: SessionViewProps) {
  const { openLink } = useSessionServices();
  const io = useTerminalIo(session.id);
  return (
    <div className={styles.view}>
      <SessionHeader session={session} summary={summary} />
      <div className={styles.terminal}>
        <Terminal io={io} onOpenLink={openLink} />
      </div>
    </div>
  );
}
