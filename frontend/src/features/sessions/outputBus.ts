import type { SessionId } from "../../ipc/sessions";

/**
 * The most output kept per session, for a terminal that mounts after the output
 * arrived (a tab opened in the background, a tab shown again): about the
 * scrollback of xterm.js at its default 1000 lines of a wide terminal.
 */
export const OUTPUT_REPLAY_BYTES = 256 * 1024;

type OutputListener = (bytes: Uint8Array) => void;

/** The output of the terminals, kept out of the store: bytes are not UI state. */
export interface OutputBus {
  /** New output of session `id`. */
  publish: (id: SessionId, bytes: Uint8Array) => void;
  /**
   * Calls `listener` with the output kept so far, then with each new one;
   * returns the function that stops it.
   */
  subscribe: (id: SessionId, listener: OutputListener) => () => void;
  /** Forgets session `id`. */
  drop: (id: SessionId) => void;
}

interface Stream {
  chunks: Uint8Array[];
  size: number;
  listeners: Set<OutputListener>;
}

/** A bus that keeps the last {@link OUTPUT_REPLAY_BYTES} of each session. */
export function createOutputBus(replayBytes = OUTPUT_REPLAY_BYTES): OutputBus {
  const streams = new Map<SessionId, Stream>();
  const streamOf = (id: SessionId): Stream => {
    let stream = streams.get(id);
    if (stream === undefined) {
      stream = { chunks: [], size: 0, listeners: new Set() };
      streams.set(id, stream);
    }
    return stream;
  };

  return {
    publish(id, bytes) {
      const stream = streamOf(id);
      stream.chunks.push(bytes);
      stream.size += bytes.length;
      // Whole chunks only: a cut escape sequence would garble the replay.
      while (stream.size > replayBytes && stream.chunks.length > 1) {
        stream.size -= stream.chunks.shift()?.length ?? 0;
      }
      stream.listeners.forEach((listener) => listener(bytes));
    },
    subscribe(id, listener) {
      const stream = streamOf(id);
      stream.chunks.forEach((chunk) => listener(chunk));
      stream.listeners.add(listener);
      return () => stream.listeners.delete(listener);
    },
    drop(id) {
      streams.delete(id);
    },
  };
}
