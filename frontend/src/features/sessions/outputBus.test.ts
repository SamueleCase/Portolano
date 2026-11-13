import { describe, expect, it } from "vitest";

import { createOutputBus } from "./outputBus";

const bytes = (text: string) => new TextEncoder().encode(text);
const text = (chunks: Uint8Array[]) => chunks.map((chunk) => new TextDecoder().decode(chunk));

describe("createOutputBus", () => {
  it("replays the output kept so far to a new listener, then sends the new one", () => {
    const bus = createOutputBus();
    const received: Uint8Array[] = [];
    bus.open("session-1");
    bus.publish("session-1", bytes("prima "));

    bus.subscribe("session-1", (chunk) => received.push(chunk));
    bus.publish("session-1", bytes("dopo"));

    expect(text(received)).toEqual(["prima ", "dopo"]);
  });

  it("keeps only the latest whole chunks within the limit", () => {
    const bus = createOutputBus({ bytes: 8, chunks: 100 });
    bus.open("session-1");
    bus.publish("session-1", bytes("vecchio"));
    bus.publish("session-1", bytes("nuovo"));
    const received: Uint8Array[] = [];

    bus.subscribe("session-1", (chunk) => received.push(chunk));

    expect(text(received)).toEqual(["nuovo"]);
  });

  it("keeps only the latest pieces within their number", () => {
    const bus = createOutputBus({ bytes: 1024, chunks: 3 });
    bus.open("session-1");
    for (const key of "abcdef") bus.publish("session-1", bytes(key));
    const received: Uint8Array[] = [];

    bus.subscribe("session-1", (chunk) => received.push(chunk));

    expect(text(received)).toEqual(["d", "e", "f"]);
  });

  it("keeps sessions apart and stops a listener on request", () => {
    const bus = createOutputBus();
    bus.open("session-1");
    bus.open("session-2");
    const received: Uint8Array[] = [];
    const stop = bus.subscribe("session-1", (chunk) => received.push(chunk));

    bus.publish("session-2", bytes("altra"));
    stop();
    bus.publish("session-1", bytes("tardi"));

    expect(received).toEqual([]);
  });

  it("drops output that arrives after the session was dropped", () => {
    const bus = createOutputBus();
    bus.open("session-1");
    bus.drop("session-1");
    const received: Uint8Array[] = [];

    bus.publish("session-1", bytes("tardi"));
    bus.subscribe("session-1", (chunk) => received.push(chunk));

    expect(received).toEqual([]);
  });

  it("keeps nothing for a session never opened", () => {
    const bus = createOutputBus();
    const received: Uint8Array[] = [];

    bus.publish("session-9", bytes("sconosciuta"));
    bus.open("session-9");
    bus.subscribe("session-9", (chunk) => received.push(chunk));

    expect(received).toEqual([]);
  });
});
