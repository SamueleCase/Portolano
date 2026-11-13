import { texts } from "../../i18n/it";
import type {
  OpenSessionRequest,
  SessionHandlers,
  SessionId,
  SessionTransport,
  TerminalSize,
} from "../../ipc/sessions";
import type { OutputBus } from "./outputBus";
import type { Session, SessionsState } from "./store";

/** A session the interface no longer knows, such as one whose tab was closed. */
export class SessionNotFound extends Error {
  readonly id: SessionId;

  constructor(id: SessionId) {
    super(texts.sessions.notFound);
    this.name = "SessionNotFound";
    this.id = id;
  }
}

/** Opens and drives sessions, keeping the store and the output bus current. */
export interface SessionController {
  /**
   * Opens a session on the connection of `request`; resolves to its id. The
   * login, if any, goes to the backend and is not kept.
   */
  open: (request: OpenSessionRequest) => Promise<SessionId>;
  input: (id: SessionId, data: string) => Promise<void>;
  resize: (id: SessionId, size: TerminalSize) => Promise<void>;
  /**
   * Ends the program of session `id` ("Disconnetti"); its tab stays, with the
   * last state the backend sends.
   */
  disconnect: (id: SessionId) => Promise<void>;
  /** Closes session `id`, ending its program if it still runs, and forgets it. */
  close: (id: SessionId) => Promise<void>;
  /**
   * Closes session `id` and opens its connection again with `again` (a login
   * asked anew, if the credential is asked at launch); resolves to the new id.
   */
  retry: (id: SessionId, again: Omit<OpenSessionRequest, "nodeId">) => Promise<SessionId>;
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
  const open = async (request: OpenSessionRequest): Promise<SessionId> => {
    const deferred = deferredHandlers();
    const opened = await transport.open(request, deferred.handlers);
    const session: Session = {
      id: opened.id,
      nodeId: request.nodeId,
      kind: opened.kind,
      state: { state: "connecting" },
      startedAt: now(),
    };
    store().added(session);
    output.open(opened.id);
    deferred.attach({
      onOutput: (bytes) => output.publish(opened.id, bytes),
      onState: (state) => store().stateChanged(opened.id, state, now()),
    });
    return opened.id;
  };

  // The sessions already closed in the backend by "Disconnetti": closing their
  // tab only forgets them.
  const released = new Set<SessionId>();

  const disconnect = async (id: SessionId): Promise<void> => {
    released.add(id);
    await transport.close(id);
  };

  const close = async (id: SessionId): Promise<void> => {
    try {
      if (!released.delete(id)) {
        await transport.close(id);
      }
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
    disconnect,
    close,
    async retry(id, again) {
      const session = store().sessions.find((candidate) => candidate.id === id);
      if (session === undefined) {
        throw new SessionNotFound(id);
      }
      await close(id);
      return open({ ...again, nodeId: session.nodeId });
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
