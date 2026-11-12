import { describe, expect, it } from "vitest";

import { createOutputBus } from "./outputBus";

const bytes = (text: string) => new TextEncoder().encode(text);
const text = (chunks: Uint8Array[]) => chunks.map((chunk) => new TextDecoder().decode(chunk));

describe("createOutputBus", () => {
  it("replays the output kept so far to a new listener, then sends the new one", () => {
    const bus = createOutputBus();
    const received: Uint8Array[] = [];
    bus.publish("session-1", bytes("prima "));

    bus.subscribe("session-1", (chunk) => received.push(chunk));
    bus.publish("session-1", bytes("dopo"));

    expect(text(received)).toEqual(["prima ", "dopo"]);
  });

  it("keeps only the latest whole chunks within the limit", () => {
    const bus = createOutputBus(8);
    bus.publish("session-1", bytes("vecchio"));
    bus.publish("session-1", bytes("nuovo"));
    const received: Uint8Array[] = [];

    bus.subscribe("session-1", (chunk) => received.push(chunk));

    expect(text(received)).toEqual(["nuovo"]);
  });

  it("keeps sessions apart and stops a listener on request", () => {
    const bus = createOutputBus();
    const received: Uint8Array[] = [];
    const stop = bus.subscribe("session-1", (chunk) => received.push(chunk));

    bus.publish("session-2", bytes("altra"));
    stop();
    bus.publish("session-1", bytes("tardi"));

    expect(received).toEqual([]);
  });

  it("forgets a dropped session", () => {
    const bus = createOutputBus();
    bus.publish("session-1", bytes("vecchio"));
    const received: Uint8Array[] = [];

    bus.drop("session-1");
    bus.subscribe("session-1", (chunk) => received.push(chunk));

    expect(received).toEqual([]);
  });
});
