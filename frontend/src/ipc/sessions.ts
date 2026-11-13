/**
 * The transport of the sessions (SPEC-UI §4.4, §5.1): how the interface opens,
 * drives and closes a session and receives its output and states. The app uses
 * the commands and channels of the bindings (`sessionsTauri.ts`); the tests a
 * fake in memory (`test/fakeSessions.ts`). The only secret that crosses it is
 * the password of a login asked at launch, on its way to the backend.
 */

import type {
  SessionKind as BackendSessionKind,
  LoginInput_Deserialize,
  SessionId,
  SessionOptionsInput_Deserialize,
  SessionState_Serialize,
  TerminalSize,
} from "../bindings";

export type { SessionId, TerminalSize };

/**
 * How a session is shown: the kinds of the backend, and the integrated RDP
 * panel to come (DEC-027), which has no view yet.
 */
export type SessionKind = BackendSessionKind | "rdp-embedded";

/** The state of a session, as the backend sends it (ambiguity 11). */
export type SessionState = SessionState_Serialize;

/** The options of a connection: the RDP client chosen in the interface. */
export type SessionOptions = SessionOptionsInput_Deserialize;

/** The login typed for a credential "ask at launch" (DEC-016.3). */
export type SessionLogin = LoginInput_Deserialize;

/** What opening a session needs. */
export interface OpenSessionRequest {
  /** The connection to open. */
  nodeId: string;
  /** The size of the terminal, for a session that turns out to have one. */
  size: TerminalSize;
  options?: SessionOptions;
  /** Only for a credential "ask at launch"; never kept after the call. */
  login?: SessionLogin;
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
