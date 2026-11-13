import { create } from "zustand";

import type { SessionId, SessionKind, SessionState } from "../../ipc/sessions";
import { exitCodeOf, isOver } from "./sessionState";

/** An open session, as the tabs show it (SPEC-UI §5.1). */
export interface Session {
  id: SessionId;
  /** The connection it opens. */
  nodeId: string;
  kind: SessionKind;
  state: SessionState;
  /** When it was opened, in milliseconds since the epoch. */
  startedAt: number;
  /** When its program was over, for a duration that stops there. */
  endedAt?: number;
  /** The exit code of its program, once over. */
  exitCode?: number;
}

/** The open sessions, in the order of their tabs. */
export interface SessionsState {
  sessions: Session[];
  /** Adds a session just opened. */
  added: (session: Session) => void;
  /** Puts `session` in the place of session `id` ("Riprova", DEC-033.3). */
  replaced: (id: SessionId, session: Session) => void;
  /**
   * Records a new state of session `id` at time `at`. Unknown sessions (a
   * closed tab, or one replaced by "Riprova"), and sessions already over (the
   * close of a session that had failed), are left as they are.
   */
  stateChanged: (id: SessionId, state: SessionState, at?: number) => void;
  /** Forgets session `id`, whose tab was closed. */
  removed: (id: SessionId) => void;
}

/** The store of the sessions; their output is not state (see `outputBus`). */
export const useSessionsStore = create<SessionsState>()((set) => ({
  sessions: [],
  added: (session) => set(({ sessions }) => ({ sessions: [...sessions, session] })),
  replaced: (id, session) =>
    set(({ sessions }) => ({
      sessions: sessions.map((current) => (current.id === id ? session : current)),
    })),
  stateChanged: (id, state, at = Date.now()) =>
    set((current) => {
      const index = current.sessions.findIndex((session) => session.id === id);
      const session = current.sessions[index];
      if (session === undefined || isOver(session.state)) {
        // The same state: nothing re-renders for a state that changes nothing.
        return current;
      }
      const sessions = [...current.sessions];
      sessions[index] = withState(session, state, at);
      return { sessions };
    }),
  removed: (id) =>
    set(({ sessions }) => ({ sessions: sessions.filter((session) => session.id !== id) })),
}));

/** `session`, still running, in `state`. */
function withState(session: Session, state: SessionState, at: number): Session {
  if (!isOver(state)) {
    return { ...session, state };
  }
  const exitCode = exitCodeOf(state);
  // With `exactOptionalPropertyTypes` an absent code is a missing key, not
  // `undefined`.
  const ended = { ...session, state, endedAt: at };
  return exitCode === undefined ? ended : { ...ended, exitCode };
}
