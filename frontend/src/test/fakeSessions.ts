import { IpcFailure } from "../ipc/result";
import type {
  SessionHandlers,
  SessionId,
  SessionKind,
  SessionOpened,
  SessionState,
  SessionTransport,
  TerminalSize,
} from "../ipc/sessions";

/** How a fake connection behaves; invented data only (SPEC §1.3). */
export interface FakeScenario {
  kind: SessionKind;
  /** Printed by a terminal once connected. */
  banner?: string;
  /** The last state, sent right after the banner; none keeps it running. */
  end?: SessionState;
}

/** The connections of the end-to-end tests, by node id. */
export const fakeScenarios: Record<string, FakeScenario> = {
  "node-ssh-ok": {
    kind: "terminal",
    banner: "Last login: Wed Nov 11 09:14:02 2026 from 192.0.2.40\r\noperatore@web01:~$ ",
  },
  "node-ssh-refused": {
    kind: "terminal",
    banner: "ssh: connect to host web02.example.com port 22: Connection refused\r\n",
    end: { state: "failed", code: 255, reason: "connessione non riuscita" },
  },
  "node-rdp": { kind: "rdp-external" },
  "node-windows-app": { kind: "external" },
};

interface FakeSession {
  kind: SessionKind;
  handlers: SessionHandlers;
  running: boolean;
}

/**
 * A transport in memory for Vitest and the end-to-end tests: a terminal prints
 * the banner of its scenario and echoes what is typed, as a remote shell would;
 * closing a running session ends it with code 0. It records every call.
 */
export class FakeSessionTransport implements SessionTransport {
  readonly inputs: { id: SessionId; data: string }[] = [];
  readonly resizes: { id: SessionId; size: TerminalSize }[] = [];
  readonly closed: SessionId[] = [];
  private readonly sessions = new Map<SessionId, FakeSession>();
  private nextId = 1;

  constructor(private readonly scenarios: Record<string, FakeScenario> = fakeScenarios) {}

  open(request: { nodeId: string }, handlers: SessionHandlers): Promise<SessionOpened> {
    const scenario = this.scenarios[request.nodeId];
    if (scenario === undefined) {
      return Promise.reject(
        new IpcFailure({ code: "notFound", message: `nodo ${request.nodeId} inesistente` }),
      );
    }
    const id = `fake-session-${this.nextId++}`;
    this.sessions.set(id, { kind: scenario.kind, handlers, running: true });
    // As the backend: "connecting" before the answer, the rest after it.
    handlers.onState({ state: "connecting" });
    setTimeout(() => this.play(id, scenario), 0);
    return Promise.resolve({ id, kind: scenario.kind });
  }

  input(id: SessionId, data: string): Promise<void> {
    this.inputs.push({ id, data });
    const session = this.sessions.get(id);
    if (session?.kind === "terminal" && session.running) {
      session.handlers.onOutput(new TextEncoder().encode(data));
    }
    return Promise.resolve();
  }

  resize(id: SessionId, size: TerminalSize): Promise<void> {
    this.resizes.push({ id, size });
    return Promise.resolve();
  }

  close(id: SessionId): Promise<void> {
    this.closed.push(id);
    const session = this.sessions.get(id);
    this.sessions.delete(id);
    if (session?.running === true) {
      this.finish(session, { state: "ended", code: 0 });
    }
    return Promise.resolve();
  }

  private play(id: SessionId, scenario: FakeScenario): void {
    const session = this.sessions.get(id);
    if (session === undefined) {
      return;
    }
    if (scenario.banner !== undefined) {
      session.handlers.onOutput(new TextEncoder().encode(scenario.banner));
    }
    session.handlers.onState({ state: "connected" });
    if (scenario.end !== undefined) {
      this.finish(session, scenario.end);
    }
  }

  private finish(session: FakeSession, state: SessionState): void {
    session.running = false;
    session.handlers.onState(state);
  }
}
