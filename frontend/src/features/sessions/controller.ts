import { texts } from "../../i18n/it";
import { IpcFailure } from "../../ipc/result";
import type {
  OpenSessionRequest,
  SessionHandlers,
  SessionId,
  SessionLogin,
  SessionOptions,
  SessionTransport,
  TerminalSize,
} from "../../ipc/sessions";
import type { OutputBus } from "./outputBus";
import type { Session, SessionsState } from "./store";

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
   * Opens the connection of session `id` again with its options (DEC-033.3):
   * the new session takes the place of the old one in the store, which is
   * then closed; resolves to the new id. `login` is a login asked anew for a
   * credential "ask at launch", not kept. If the new session cannot open,
   * the old one stays and the promise rejects.
   */
  retry: (id: SessionId, size: TerminalSize, login?: SessionLogin) => Promise<SessionId>;
}

/** What the controller works with; `now` is a parameter for the tests. */
export interface SessionControllerDeps {
  transport: SessionTransport;
  /** The store's state (`useSessionsStore.getState`). */
  store: () => SessionsState;
  output: OutputBus;
  now?: () => number;
}

/** A session that started, before it reaches the store. */
interface Started {
  session: Session;
  /** Sends what arrived meanwhile, and what comes next, to the store and the bus. */
  attach: () => void;
}

/** A controller over `deps`. */
export function createSessionController({
  transport,
  store,
  output,
  now = Date.now,
}: SessionControllerDeps): SessionController {
  // The options of each session, for "Riprova": never secret, unlike a login.
  const optionsOf = new Map<SessionId, SessionOptions | undefined>();
  // The sessions already closed in the backend by "Disconnetti": closing their
  // tab only forgets them.
  const released = new Set<SessionId>();

  const start = async (request: OpenSessionRequest): Promise<Started> => {
    const deferred = deferredHandlers();
    const opened = await transport.open(request, deferred.handlers);
    optionsOf.set(opened.id, request.options);
    const session: Session = {
      id: opened.id,
      nodeId: request.nodeId,
      kind: opened.kind,
      state: { state: "connecting" },
      startedAt: now(),
    };
    return {
      session,
      attach: () => {
        // The stream first: output may come as soon as the tab shows.
        output.open(opened.id);
        deferred.attach({
          onOutput: (bytes) => output.publish(opened.id, bytes),
          onState: (state) => store().stateChanged(opened.id, state, now()),
        });
      },
    };
  };

  /** Ends session `id` in the backend, unless "Disconnetti" already did. */
  const release = async (id: SessionId): Promise<void> => {
    if (!released.delete(id)) {
      await transport.close(id);
    }
  };

  /** Forgets session `id` in the interface. */
  const forget = (id: SessionId) => {
    optionsOf.delete(id);
    output.drop(id);
  };

  return {
    async open(request) {
      const { session, attach } = await start(request);
      store().added(session);
      attach();
      return session.id;
    },
    input: (id, data) => transport.input(id, data),
    resize: (id, size) => transport.resize(id, size),
    async disconnect(id) {
      released.add(id);
      try {
        await transport.close(id);
      } catch (error) {
        // Still running: closing the tab must end it.
        released.delete(id);
        throw error;
      }
    },
    async close(id) {
      try {
        await release(id);
      } finally {
        // The tab goes away even if the backend no longer knows the session.
        store().removed(id);
        forget(id);
      }
    },
    async retry(id, size, login) {
      const previous = store().sessions.find((session) => session.id === id);
      if (previous === undefined) {
        throw new IpcFailure({ code: "notFound", message: texts.sessions.notFound });
      }
      const request: OpenSessionRequest = { nodeId: previous.nodeId, size };
      const options = optionsOf.get(id);
      if (options !== undefined) request.options = options;
      if (login !== undefined) request.login = login;
      const { session, attach } = await start(request);
      store().replaced(id, session);
      attach();
      forget(id);
      // The old program is over or about to be: a failure to end it leaves
      // nothing to show, its tab is already the new one.
      await release(id).catch(() => undefined);
      return session.id;
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
