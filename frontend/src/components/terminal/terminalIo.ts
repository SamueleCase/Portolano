/**
 * What the terminal needs from a session, and nothing else: UI-WP3 binds it
 * to the IPC channel of a PTY, the gallery and the tests to a fake. The
 * terminal knows neither IPC nor sessions.
 */
export interface TerminalIo {
  /** Calls `onOutput` with each chunk of raw output; returns the unsubscription. */
  subscribe(onOutput: (data: Uint8Array) => void): () => void;
  /** What the user typed or pasted. */
  input(data: string): void;
  /** The new size of the terminal, in cells. */
  resize(columns: number, rows: number): void;
}
