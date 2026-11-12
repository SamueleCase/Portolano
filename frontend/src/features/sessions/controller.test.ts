import { beforeEach, describe, expect, it, vi } from "vitest";

import { IpcFailure } from "../../ipc/result";
import { FakeSessionTransport } from "../../test/fakeSessions";
import { createSessionController } from "./controller";
import { createOutputBus } from "./outputBus";
import { useSessionsStore } from "./store";

const SIZE = { cols: 80, rows: 24 };

function setup() {
  const transport = new FakeSessionTransport();
  const output = createOutputBus();
  const controller = createSessionController({
    transport,
    store: useSessionsStore.getState,
    output,
    now: () => 42_000,
  });
  return { transport, output, controller };
}

const sessions = () => useSessionsStore.getState().sessions;

/** Lets the fake transport play its scenario. */
const played = () => vi.waitFor(() => expect(sessions()[0]?.state.state).not.toBe("connecting"));

describe("createSessionController", () => {
  beforeEach(() => {
    useSessionsStore.setState({ sessions: [] });
  });

  it("adds an opened session with its kind and start time", async () => {
    const { controller } = setup();

    const id = await controller.open("node-rdp", SIZE);

    expect(sessions()).toEqual([
      {
        id,
        nodeId: "node-rdp",
        kind: "rdp-external",
        state: { state: "connecting" },
        startedAt: 42_000,
      },
    ]);
  });

  it("follows the states of the session", async () => {
    const { controller } = setup();

    await controller.open("node-ssh-ok", SIZE);
    await played();

    expect(sessions()[0]?.state).toEqual({ state: "connected" });
  });

  it("records a failed connection with its exit code", async () => {
    const { controller } = setup();

    await controller.open("node-ssh-refused", SIZE);
    await vi.waitFor(() => expect(sessions()[0]?.state.state).toBe("failed"));

    expect(sessions()[0]?.exitCode).toBe(255);
  });

  it("sends the output of the terminal to the bus", async () => {
    const { controller, output } = setup();
    const id = await controller.open("node-ssh-ok", SIZE);
    await played();
    const received: string[] = [];

    output.subscribe(id, (bytes) => received.push(new TextDecoder().decode(bytes)));

    expect(received.join("")).toContain("operatore@web01:~$");
  });

  it("passes input and size to the transport", async () => {
    const { controller, transport } = setup();
    const id = await controller.open("node-ssh-ok", SIZE);

    await controller.input(id, "ls\r");
    await controller.resize(id, { cols: 120, rows: 40 });

    expect(transport.inputs).toEqual([{ id, data: "ls\r" }]);
    expect(transport.resizes).toEqual([{ id, size: { cols: 120, rows: 40 } }]);
  });

  it("closes and forgets a session", async () => {
    const { controller, transport } = setup();
    const id = await controller.open("node-ssh-ok", SIZE);

    await controller.close(id);

    expect(transport.closed).toEqual([id]);
    expect(sessions()).toEqual([]);
  });

  it("retries a failed connection as a new session on the same node", async () => {
    const { controller } = setup();
    const failed = await controller.open("node-ssh-refused", SIZE);

    const retried = await controller.retry(failed, SIZE);

    expect(retried).not.toBe(failed);
    expect(sessions().map(({ id, nodeId }) => ({ id, nodeId }))).toEqual([
      { id: retried, nodeId: "node-ssh-refused" },
    ]);
  });

  it("keeps what arrives before the session has its id", async () => {
    const output = createOutputBus();
    const transport = new FakeSessionTransport();
    transport.open = (_request, handlers) => {
      handlers.onOutput(new TextEncoder().encode("presto"));
      handlers.onState({ state: "connected" });
      return Promise.resolve({ id: "session-early", kind: "terminal" });
    };
    const controller = createSessionController({
      transport,
      store: useSessionsStore.getState,
      output,
    });
    const received: string[] = [];

    await controller.open("node-ssh-ok", SIZE);
    output.subscribe("session-early", (bytes) => received.push(new TextDecoder().decode(bytes)));

    expect(sessions()[0]?.state).toEqual({ state: "connected" });
    expect(received).toEqual(["presto"]);
  });

  it("rejects the opening of an unknown connection without adding a session", async () => {
    const { controller } = setup();

    await expect(controller.open("node-missing", SIZE)).rejects.toBeInstanceOf(IpcFailure);
    expect(sessions()).toEqual([]);
  });
});
