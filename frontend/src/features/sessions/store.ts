import { create } from "zustand";

import type { SessionId, SessionKind, SessionState } from "../../ipc/sessions";
import { exitCodeOf } from "./sessionState";

/** An open session, as the tabs show it (SPEC-UI §5.1). */
export interface Session {
  id: SessionId;
  /** The connection it opens. */
  nodeId: string;
  kind: SessionKind;
  state: SessionState;
  /** When it was opened, in milliseconds since the epoch. */
  startedAt: number;
  /** The exit code of its program, once over. */
  exitCode?: number;
}

/** The open sessions, in the order they were opened. */
export interface SessionsState {
  sessions: Session[];
  /** Adds a session just opened. */
  added: (session: Session) => void;
  /** Records a new state of session `id`; unknown sessions are ignored. */
  stateChanged: (id: SessionId, state: SessionState) => void;
  /** Forgets session `id`, whose tab was closed. */
  removed: (id: SessionId) => void;
}

/** The store of the sessions; their output is not state (see `outputBus`). */
export const useSessionsStore = create<SessionsState>()((set) => ({
  sessions: [],
  added: (session) => set(({ sessions }) => ({ sessions: [...sessions, session] })),
  stateChanged: (id, state) =>
    set(({ sessions }) => ({
      sessions: sessions.map((session) =>
        session.id === id ? withState(session, state) : session,
      ),
    })),
  removed: (id) =>
    set(({ sessions }) => ({ sessions: sessions.filter((session) => session.id !== id) })),
}));

function withState(session: Session, state: SessionState): Session {
  const exitCode = exitCodeOf(state);
  // With `exactOptionalPropertyTypes` an absent code is a missing key, not
  // `undefined`; states only move forward, so a code is never taken away.
  return exitCode === undefined ? { ...session, state } : { ...session, state, exitCode };
}
