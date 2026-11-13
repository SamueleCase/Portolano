import type { SessionId, SessionTransport, TerminalSize } from "./sessions";

/** A call waiting for its turn, with the promises it settles. */
type Pending =
  | { kind: "input"; data: string; waiters: Waiter[] }
  | { kind: "resize"; size: TerminalSize; waiters: Waiter[] };

interface Waiter {
  resolve: () => void;
  reject: (error: unknown) => void;
}

/**
 * `transport` with the input and the sizes of each session sent one call at a
 * time, in order (DEC-033.2). Each IPC call runs on its own, so two keys typed
 * quickly could reach the terminal swapped; here what is typed while a call
 * runs is joined into the next one, and of the sizes only the last counts.
 */
export function serializedTransport(transport: SessionTransport): SessionTransport {
  const queues = new Map<SessionId, Pending[]>();

  const drain = async (id: SessionId, queue: Pending[]): Promise<void> => {
    for (let next = queue[0]; next !== undefined; next = queue[0]) {
      try {
        await (next.kind === "input"
          ? transport.input(id, next.data)
          : transport.resize(id, next.size));
        next.waiters.forEach((waiter) => waiter.resolve());
      } catch (error) {
        next.waiters.forEach((waiter) => waiter.reject(error));
      }
      queue.shift();
    }
    queues.delete(id);
  };

  const enqueue = (id: SessionId, add: (queue: Pending[], waiter: Waiter) => void) =>
    new Promise<void>((resolve, reject) => {
      const queue = queues.get(id);
      if (queue === undefined) {
        const started: Pending[] = [];
        queues.set(id, started);
        add(started, { resolve, reject });
        void drain(id, started);
      } else {
        add(queue, { resolve, reject });
      }
    });

  return {
    open: (request, handlers) => transport.open(request, handlers),
    close: (id) => transport.close(id),
    input: (id, data) =>
      enqueue(id, (queue, waiter) => {
        // The first entry is running: only a later one may still grow.
        const last = queue.length > 1 ? queue.at(-1) : undefined;
        if (last?.kind === "input") {
          last.data += data;
          last.waiters.push(waiter);
        } else {
          queue.push({ kind: "input", data, waiters: [waiter] });
        }
      }),
    resize: (id, size) =>
      enqueue(id, (queue, waiter) => {
        const last = queue.length > 1 ? queue.at(-1) : undefined;
        if (last?.kind === "resize") {
          last.size = size;
          last.waiters.push(waiter);
        } else {
          queue.push({ kind: "resize", size, waiters: [waiter] });
        }
      }),
  };
}
