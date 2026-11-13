import { describe, expect, it } from "vitest";

import type { SessionTransport, TerminalSize } from "./sessions";
import { serializedTransport } from "./sessionsQueue";

/** A transport whose calls finish only when the test says so. */
function slowTransport() {
  const calls: string[] = [];
  const running: (() => void)[] = [];
  let concurrent = 0;
  let mostConcurrent = 0;
  const call = (what: string) =>
    new Promise<void>((resolve) => {
      calls.push(what);
      concurrent += 1;
      mostConcurrent = Math.max(mostConcurrent, concurrent);
      running.push(() => {
        concurrent -= 1;
        resolve();
      });
    });
  const transport: SessionTransport = {
    open: () => Promise.resolve({ id: "session-1", kind: "terminal" }),
    close: () => Promise.resolve(),
    input: (id, data) => call(`${id} input ${data}`),
    resize: (id, size: TerminalSize) => call(`${id} resize ${size.cols}x${size.rows}`),
  };
  return {
    transport,
    calls,
    mostConcurrent: () => mostConcurrent,
    /** Finishes the calls that run now, and lets the next ones start. */
    async finish() {
      running.splice(0).forEach((done) => done());
      await new Promise((resolve) => setTimeout(resolve, 0));
    },
  };
}

describe("serializedTransport", () => {
  it("sends many concurrent inputs in order, one call at a time", async () => {
    const slow = slowTransport();
    const transport = serializedTransport(slow.transport);
    const keys = [..."ls -la\r"];

    const sent = keys.map((key) => transport.input("session-1", key));
    while (slow.calls.length < 2) await slow.finish();
    await slow.finish();
    await Promise.all(sent);

    expect(slow.calls).toEqual(["session-1 input l", "session-1 input s -la\r"]);
    expect(slow.mostConcurrent()).toBe(1);
  });

  it("keeps only the last size waiting", async () => {
    const slow = slowTransport();
    const transport = serializedTransport(slow.transport);

    const sent = [
      transport.resize("session-1", { cols: 80, rows: 24 }),
      transport.resize("session-1", { cols: 100, rows: 30 }),
      transport.resize("session-1", { cols: 120, rows: 40 }),
    ];
    await slow.finish();
    await slow.finish();
    await Promise.all(sent);

    expect(slow.calls).toEqual(["session-1 resize 80x24", "session-1 resize 120x40"]);
  });

  it("keeps input and sizes in the order they came", async () => {
    const slow = slowTransport();
    const transport = serializedTransport(slow.transport);

    const sent = [
      transport.input("session-1", "a"),
      transport.input("session-1", "b"),
      transport.resize("session-1", { cols: 90, rows: 20 }),
      transport.input("session-1", "c"),
    ];
    for (let round = 0; round < 4; round += 1) await slow.finish();
    await Promise.all(sent);

    expect(slow.calls).toEqual([
      "session-1 input a",
      "session-1 input b",
      "session-1 resize 90x20",
      "session-1 input c",
    ]);
  });

  it("lets two sessions run side by side", async () => {
    const slow = slowTransport();
    const transport = serializedTransport(slow.transport);

    const sent = [transport.input("session-1", "a"), transport.input("session-2", "b")];
    await slow.finish();
    await Promise.all(sent);

    expect(slow.mostConcurrent()).toBe(2);
  });

  it("rejects every caller of a failed call, and goes on with the next ones", async () => {
    const calls: string[] = [];
    const transport = serializedTransport({
      open: () => Promise.resolve({ id: "session-1", kind: "terminal" }),
      close: () => Promise.resolve(),
      resize: () => Promise.resolve(),
      input: (_id, data) => {
        calls.push(data);
        return data === "x" ? Promise.reject(new Error("persa")) : Promise.resolve();
      },
    });

    const failed = transport.input("session-1", "x");
    const next = transport.input("session-1", "y");

    await expect(failed).rejects.toThrow("persa");
    await expect(next).resolves.toBeUndefined();
    expect(calls).toEqual(["x", "y"]);
  });
});
