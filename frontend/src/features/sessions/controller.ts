import type {
  SessionHandlers,
  SessionId,
  SessionTransport,
  TerminalSize,
} from "../../ipc/sessions";
import type { OutputBus } from "./outputBus";
import type { Session, SessionsState } from "./store";

/** Opens and drives sessions, keeping the store and the output bus current. */
export interface SessionController {
  /** Opens a session on connection `nodeId`; resolves to its id. */
  open: (nodeId: string, size: TerminalSize) => Promise<SessionId>;
  input: (id: SessionId, data: string) => Promise<void>;
  resize: (id: SessionId, size: TerminalSize) => Promise<void>;
  /** Closes session `id` and forgets it. */
  close: (id: SessionId) => Promise<void>;
  /** Closes session `id` and opens its connection again; resolves to the new id. */
  retry: (id: SessionId, size: TerminalSize) => Promise<SessionId>;
}

/** What the controller works with; `now` is a parameter for the tests. */
export interface SessionControllerDeps {
  transport: SessionTransport;
  /** The store's state (`useSessionsStore.getState`). */
  store: () => SessionsState;
  output: OutputBus;
  now?: () => number;
}

/** A controller over `deps`. */
export function createSessionController({
  transport,
  store,
  output,
  now = Date.now,
}: SessionControllerDeps): SessionController {
  const open = async (nodeId: string, size: TerminalSize): Promise<SessionId> => {
    const deferred = deferredHandlers();
    const opened = await transport.open({ nodeId, size }, deferred.handlers);
    const session: Session = {
      id: opened.id,
      nodeId,
      kind: opened.kind,
      state: { state: "connecting" },
      startedAt: now(),
    };
    store().added(session);
    deferred.attach({
      onOutput: (bytes) => output.publish(opened.id, bytes),
      onState: (state) => store().stateChanged(opened.id, state),
    });
    return opened.id;
  };

  const close = async (id: SessionId): Promise<void> => {
    try {
      await transport.close(id);
    } finally {
      // The tab goes away even if the backend no longer knows the session.
      store().removed(id);
      output.drop(id);
    }
  };

  return {
    open,
    input: (id, data) => transport.input(id, data),
    resize: (id, size) => transport.resize(id, size),
    close,
    async retry(id, size) {
      const session = store().sessions.find((candidate) => candidate.id === id);
      if (session === undefined) {
        throw new Error(`unknown session ${id}`);
      }
      await close(id);
      return open(session.nodeId, size);
    },
  };
}

/**
 * Handlers that hold what arrives before the session has its id (the backend
 * reports "connecting" while the command runs) and pass it on, in order, once
 * attached to their target.
 */
function deferredHandlers(): {
  handlers: SessionHandlers;
  attach: (target: SessionHandlers) => void;
} {
  let target: SessionHandlers | undefined;
  const waiting: ((handlers: SessionHandlers) => void)[] = [];
  const deliver = (send: (handlers: SessionHandlers) => void) => {
    if (target === undefined) {
      waiting.push(send);
    } else {
      send(target);
    }
  };
  return {
    handlers: {
      onOutput: (bytes) => deliver((handlers) => handlers.onOutput(bytes)),
      onState: (state) => deliver((handlers) => handlers.onState(state)),
    },
    attach(handlers) {
      target = handlers;
      waiting.splice(0).forEach((send) => send(handlers));
    },
  };
}
