import { IpcFailure } from "../ipc/result";
import type {
  OpenSessionRequest,
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
  /**
   * Only the first `failures` openings end with `end`; the next ones keep
   * running, as a server that is back ("Riprova").
   */
  failures?: number;
}

/** What a shell of the fake prints as its prompt. */
const PROMPT = "operatore@web01:~$ ";

/** What a shell of the fake prints once connected. */
const LOGIN_BANNER = `Last login: Wed Nov 11 09:14:02 2026 from 192.0.2.40\r\n${PROMPT}`;

/** The connections of the tests and of the gallery, by node id. */
export const fakeScenarios: Record<string, FakeScenario> = {
  "node-ssh-ok": {
    kind: "terminal",
    banner: LOGIN_BANNER,
  },
  "node-ssh-refused": {
    kind: "terminal",
    banner: "ssh: connect to host web02.example.com port 22: Connection refused\r\n",
    end: { state: "failed", code: 255, reason: "connessione non riuscita" },
    failures: 1,
  },
  "node-rdp": { kind: "rdp-external" },
  "node-rdp-failed": {
    kind: "rdp-external",
    // A log carries text of the server: here an escape sequence.
    end: {
      state: "failed",
      code: 131,
      reason: "[ERROR][com.freerdp.core] connessione a 192.0.2.10 rifiutata \u001b[31m",
    },
  },
  "node-windows-app": { kind: "external" },
};

interface FakeSession {
  kind: SessionKind;
  handlers: SessionHandlers;
  running: boolean;
  /** What was typed since the last line end. */
  line: string;
}

/** `exit` or `exit N` typed in a terminal ends its session with that code. */
const EXIT = /^exit(?:\s+(\d+))?$/;

/**
 * A transport in memory for Vitest, the gallery and the end-to-end tests: a
 * terminal prints the banner of its scenario and echoes what is typed, as a
 * remote shell would, and `exit N` ends it with code N; closing a running
 * session ends it with code 0. It records every call.
 */
export class FakeSessionTransport implements SessionTransport {
  readonly inputs: { id: SessionId; data: string }[] = [];
  readonly resizes: { id: SessionId; size: TerminalSize }[] = [];
  readonly closed: SessionId[] = [];
  private readonly sessions = new Map<SessionId, FakeSession>();
  private readonly attempts = new Map<string, number>();
  private nextId = 1;

  constructor(private readonly scenarios: Record<string, FakeScenario> = fakeScenarios) {}

  open(request: OpenSessionRequest, handlers: SessionHandlers): Promise<SessionOpened> {
    const scenario = this.scenarios[request.nodeId];
    if (scenario === undefined) {
      return Promise.reject(
        new IpcFailure({ code: "notFound", message: `nodo ${request.nodeId} inesistente` }),
      );
    }
    const attempt = (this.attempts.get(request.nodeId) ?? 0) + 1;
    this.attempts.set(request.nodeId, attempt);
    const id = `fake-session-${this.nextId++}`;
    this.sessions.set(id, { kind: scenario.kind, handlers, running: true, line: "" });
    // As the backend: "connecting" before the answer, the rest after it.
    handlers.onState({ state: "connecting" });
    setTimeout(() => this.play(id, scenario, attempt), 0);
    return Promise.resolve({ id, kind: scenario.kind });
  }

  input(id: SessionId, data: string): Promise<void> {
    this.inputs.push({ id, data });
    const session = this.sessions.get(id);
    if (session?.kind === "terminal" && session.running) {
      this.type(session, data);
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

  private play(id: SessionId, scenario: FakeScenario, attempt: number): void {
    const session = this.sessions.get(id);
    if (session === undefined) {
      return;
    }
    const fails = scenario.end !== undefined && attempt <= (scenario.failures ?? Infinity);
    // A server that is back greets as any other.
    const banner =
      scenario.end === undefined || fails
        ? scenario.banner
        : scenario.kind === "terminal"
          ? LOGIN_BANNER
          : undefined;
    // As the backend: "connected" comes with the first output, before it.
    session.handlers.onState({ state: "connected" });
    if (banner !== undefined) {
      session.handlers.onOutput(encode(banner));
    }
    if (fails && scenario.end !== undefined) {
      this.finish(session, scenario.end);
    }
  }

  /** Echoes `data` and runs the lines it ends. */
  private type(session: FakeSession, data: string): void {
    for (const character of data) {
      if (character !== "\r") {
        session.line += character;
        session.handlers.onOutput(encode(character));
        continue;
      }
      const exit = EXIT.exec(session.line.trim());
      session.line = "";
      session.handlers.onOutput(encode("\r\n"));
      if (exit) {
        this.finish(session, { state: "ended", code: Number(exit[1] ?? 0) });
        return;
      }
      session.handlers.onOutput(encode(PROMPT));
    }
  }

  private finish(session: FakeSession, state: SessionState): void {
    session.running = false;
    session.handlers.onState(state);
  }
}

function encode(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}
