import type { SessionId } from "../../ipc/sessions";

/**
 * The most output kept per session, for a terminal that mounts after the output
 * arrived (a tab opened in the background, a tab shown again): about the
 * scrollback of xterm.js at its default 1000 lines of a wide terminal.
 */
export const OUTPUT_REPLAY_BYTES = 256 * 1024;

/**
 * The most pieces kept per session: a program that writes one byte at a time
 * would otherwise keep a quarter of a million of them.
 */
export const OUTPUT_REPLAY_CHUNKS = 1024;

type OutputListener = (bytes: Uint8Array) => void;

/**
 * The output of the terminals, kept out of the store: bytes are not UI state.
 * Only opened sessions have output: what arrives for another id (late output of
 * a closed session) is dropped, so nothing outlives the tab.
 */
export interface OutputBus {
  /** Starts keeping the output of session `id`. */
  open: (id: SessionId) => void;
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

/**
 * The output kept for a session: `chunks` from index `first` on. Dropping the
 * oldest piece only moves `first`, so a publication costs the same however
 * many pieces are kept; the array is compacted once half of it is dropped.
 */
interface Stream {
  chunks: Uint8Array[];
  first: number;
  size: number;
  listeners: Set<OutputListener>;
}

/** The limits of what a bus keeps per session. */
interface ReplayLimits {
  bytes: number;
  chunks: number;
}

/** A bus that keeps the last output of each session, within `limits`. */
export function createOutputBus(
  limits: ReplayLimits = { bytes: OUTPUT_REPLAY_BYTES, chunks: OUTPUT_REPLAY_CHUNKS },
): OutputBus {
  const streams = new Map<SessionId, Stream>();

  const keep = (stream: Stream, bytes: Uint8Array) => {
    stream.chunks.push(bytes);
    stream.size += bytes.length;
    // Whole pieces only: a cut escape sequence would garble the replay.
    while (
      (stream.size > limits.bytes || stream.chunks.length - stream.first > limits.chunks) &&
      stream.chunks.length - stream.first > 1
    ) {
      stream.size -= stream.chunks[stream.first]?.length ?? 0;
      stream.first += 1;
    }
    if (stream.first * 2 > stream.chunks.length) {
      stream.chunks = stream.chunks.slice(stream.first);
      stream.first = 0;
    }
  };

  return {
    open(id) {
      if (!streams.has(id)) {
        streams.set(id, { chunks: [], first: 0, size: 0, listeners: new Set() });
      }
    },
    publish(id, bytes) {
      const stream = streams.get(id);
      if (stream === undefined) {
        return;
      }
      keep(stream, bytes);
      stream.listeners.forEach((listener) => listener(bytes));
    },
    subscribe(id, listener) {
      const stream = streams.get(id);
      if (stream === undefined) {
        return () => undefined;
      }
      stream.chunks.slice(stream.first).forEach((chunk) => listener(chunk));
      stream.listeners.add(listener);
      return () => stream.listeners.delete(listener);
    },
    drop(id) {
      streams.delete(id);
    },
  };
}
