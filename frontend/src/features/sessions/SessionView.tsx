import type { ComponentType } from "react";

import { texts } from "../../i18n/it";
import type { SessionKind } from "../../ipc/sessions";
import { sessionViews } from "./sessionViews";
import type { Session } from "./store";

/** What every session view receives. */
export interface SessionViewProps {
  session: Session;
  /** The summary of the connection, such as `user@host:22`: untrusted text. */
  summary: string;
}

/**
 * The view of each kind of session: the tab picks it from here and knows
 * neither xterm.js nor a canvas (SPEC-UI §5.1). A kind without a view yet (the
 * integrated RDP panel) is shown as not available.
 */
export type SessionViews = Partial<Record<SessionKind, ComponentType<SessionViewProps>>>;

interface SessionViewHostProps extends SessionViewProps {
  /** The views of the kinds; the app ones unless a test passes others. */
  views?: SessionViews;
}

/** The body of the tab of `session`, from `views`. */
export function SessionView({ session, summary, views = sessionViews }: SessionViewHostProps) {
  const View = views[session.kind];
  if (View === undefined) {
    return <p role="status">{texts.sessions.missingView}</p>;
  }
  return <View session={session} summary={summary} />;
}
