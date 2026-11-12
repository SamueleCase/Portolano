/**
 * The transport of the sessions (SPEC-UI §4.4, §5.1): how the interface opens,
 * drives and closes a session and receives its output and states. The app uses
 * the Tauri channels (`sessionsTauri.ts`); the tests a fake in memory
 * (`test/fakeSessions.ts`). Nothing that crosses it is a secret.
 */

/** Identifies an open session. */
export type SessionId = string;

/**
 * How a session is shown: a terminal (SSH, or FreeRDP that has to ask), FreeRDP
 * in its own window, an application that has the connection (Windows App), and
 * the integrated RDP panel to come (DEC-027).
 */
export type SessionKind = "terminal" | "rdp-external" | "external" | "rdp-embedded";

/** The state of a session, as the backend sends it (ambiguity 11). */
export type SessionState =
  | { state: "connecting" }
  | { state: "connected" }
  | { state: "ended"; code: number }
  | { state: "failed"; code: number | null; reason: string };

/** The size of a terminal, in characters. */
export interface TerminalSize {
  cols: number;
  rows: number;
}

/** What opening a session needs. */
export interface OpenSessionRequest {
  /** The connection to open. */
  nodeId: string;
  /** The size of the terminal, for a session that turns out to have one. */
  size: TerminalSize;
}

/** A session just opened: its kind picks the view of its tab. */
export interface SessionOpened {
  id: SessionId;
  kind: SessionKind;
}

/**
 * Receives what a session sends. Messages may arrive before `open` resolves:
 * the backend reports "connecting" while it starts the program.
 */
export interface SessionHandlers {
  /** Bytes written by the program of a terminal, for xterm.js. */
  onOutput: (bytes: Uint8Array) => void;
  /** A new state. */
  onState: (state: SessionState) => void;
}

/** Opens, drives and closes sessions. Failed calls reject with an `IpcFailure`. */
export interface SessionTransport {
  open: (request: OpenSessionRequest, handlers: SessionHandlers) => Promise<SessionOpened>;
  /** Sends what the user typed. */
  input: (id: SessionId, data: string) => Promise<void>;
  resize: (id: SessionId, size: TerminalSize) => Promise<void>;
  /** Ends the program if it still runs; its last state follows on its handlers. */
  close: (id: SessionId) => Promise<void>;
}

/** Version of the header of the output frames; other versions are ignored. */
export const FRAME_VERSION = 1;

/** What the payload of a frame holds. */
export const FrameKind = {
  terminalOutput: 0,
} as const;

/** An output frame without its header. */
export interface Frame {
  kind: number;
  payload: Uint8Array;
}

/**
 * Splits an output frame, `[version, kind, …payload]`, into kind and payload;
 * `null` for a frame of another version or without header.
 */
export function decodeFrame(buffer: ArrayBuffer): Frame | null {
  const bytes = new Uint8Array(buffer);
  if (bytes.length < 2 || bytes[0] !== FRAME_VERSION) {
    return null;
  }
  return { kind: bytes[1] ?? -1, payload: bytes.subarray(2) };
}
