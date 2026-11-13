import { beforeEach, describe, expect, it, vi } from "vitest";

import { IpcFailure } from "../../ipc/result";
import type { SessionHandlers } from "../../ipc/sessions";
import { FakeSessionTransport } from "../../test/fakeSessions";
import { texts } from "../../i18n/it";
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

    const id = await controller.open({ nodeId: "node-rdp", size: SIZE });

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

    await controller.open({ nodeId: "node-ssh-ok", size: SIZE });
    await played();

    expect(sessions()[0]?.state).toEqual({ state: "connected" });
  });

  it("records a failed connection with its exit code", async () => {
    const { controller } = setup();

    await controller.open({ nodeId: "node-ssh-refused", size: SIZE });
    await vi.waitFor(() => expect(sessions()[0]?.state.state).toBe("failed"));

    expect(sessions()[0]?.exitCode).toBe(255);
  });

  it("sends the output of the terminal to the bus", async () => {
    const { controller, output } = setup();
    const id = await controller.open({ nodeId: "node-ssh-ok", size: SIZE });
    await played();
    const received: string[] = [];

    output.subscribe(id, (bytes) => received.push(new TextDecoder().decode(bytes)));

    expect(received.join("")).toContain("operatore@web01:~$");
  });

  it("passes input and size to the transport", async () => {
    const { controller, transport } = setup();
    const id = await controller.open({ nodeId: "node-ssh-ok", size: SIZE });

    await controller.input(id, "ls\r");
    await controller.resize(id, { cols: 120, rows: 40 });

    expect(transport.inputs).toEqual([{ id, data: "ls\r" }]);
    expect(transport.resizes).toEqual([{ id, size: { cols: 120, rows: 40 } }]);
  });

  it("closes and forgets a session", async () => {
    const { controller, transport } = setup();
    const id = await controller.open({ nodeId: "node-ssh-ok", size: SIZE });

    await controller.close(id);

    expect(transport.closed).toEqual([id]);
    expect(sessions()).toEqual([]);
  });

  it("disconnects a session and keeps its tab with the last state", async () => {
    const { controller, transport } = setup();
    const id = await controller.open({ nodeId: "node-rdp", size: SIZE });

    await controller.disconnect(id);

    expect(transport.closed).toEqual([id]);
    expect(sessions()[0]).toMatchObject({
      id,
      state: { state: "ended", code: 0 },
      endedAt: 42_000,
    });
  });

  it("closes a disconnected tab without asking the backend again", async () => {
    const { controller, transport } = setup();
    const id = await controller.open({ nodeId: "node-rdp", size: SIZE });
    await controller.disconnect(id);

    await controller.close(id);

    expect(transport.closed).toEqual([id]);
    expect(sessions()).toEqual([]);
  });

  it("retries in the place of the old tab, with the same options", async () => {
    const { controller, transport } = setup();
    const opened = vi.spyOn(transport, "open");
    await controller.open({ nodeId: "node-ssh-ok", size: SIZE });
    const failed = await controller.open({
      nodeId: "node-ssh-refused",
      size: SIZE,
      options: { rdpClient: "freerdp" },
    });
    await controller.open({ nodeId: "node-rdp", size: SIZE });

    const retried = await controller.retry(failed, SIZE);

    expect(sessions().map(({ nodeId }) => nodeId)).toEqual([
      "node-ssh-ok",
      "node-ssh-refused",
      "node-rdp",
    ]);
    expect(sessions()[1]?.id).toBe(retried);
    expect(opened.mock.calls.at(-1)?.[0]).toEqual({
      nodeId: "node-ssh-refused",
      size: SIZE,
      options: { rdpClient: "freerdp" },
    });
    expect(transport.closed).toEqual([failed]);
  });

  it("keeps the old tab when the connection cannot open again", async () => {
    const { controller, transport } = setup();
    const failed = await controller.open({ nodeId: "node-ssh-refused", size: SIZE });
    transport.open = () =>
      Promise.reject(new IpcFailure({ code: "vaultLocked", message: "vault bloccato" }));

    await expect(controller.retry(failed, SIZE)).rejects.toBeInstanceOf(IpcFailure);

    expect(sessions().map(({ id }) => id)).toEqual([failed]);
    expect(transport.closed).toEqual([]);
  });

  it("passes a login to the backend without keeping it", async () => {
    const { controller, transport } = setup();
    const opened = vi.spyOn(transport, "open");
    const login = { username: "operatore", domain: null, password: "digitata-sentinella" };
    const first = await controller.open({ nodeId: "node-ssh-refused", size: SIZE, login });

    await controller.retry(first, SIZE);

    expect(opened.mock.calls[0]?.[0].login).toEqual(login);
    expect(opened.mock.calls[1]?.[0]).not.toHaveProperty("login");
    expect(JSON.stringify(useSessionsStore.getState())).not.toContain("digitata-sentinella");
  });

  it("can close a tab whose disconnection failed", async () => {
    const { controller, transport } = setup();
    const id = await controller.open({ nodeId: "node-rdp", size: SIZE });
    const close = vi
      .spyOn(transport, "close")
      .mockRejectedValueOnce(new IpcFailure({ code: "io", message: "non riuscito" }));

    await expect(controller.disconnect(id)).rejects.toBeInstanceOf(IpcFailure);
    await controller.close(id);

    expect(close).toHaveBeenCalledTimes(2);
    expect(sessions()).toEqual([]);
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

    await controller.open({ nodeId: "node-ssh-ok", size: SIZE });
    output.subscribe("session-early", (bytes) => received.push(new TextDecoder().decode(bytes)));

    expect(sessions()[0]?.state).toEqual({ state: "connected" });
    expect(received).toEqual(["presto"]);
  });

  it("keeps nothing that arrives after the session was closed", async () => {
    const { controller, output, transport } = setup();
    let late: SessionHandlers | undefined;
    transport.open = (_request, handlers) => {
      late = handlers;
      return Promise.resolve({ id: "session-late", kind: "terminal" });
    };
    await controller.open({ nodeId: "node-ssh-ok", size: SIZE });
    await controller.close("session-late");
    const received: Uint8Array[] = [];

    late?.onOutput(new TextEncoder().encode("tardi"));
    late?.onState({ state: "ended", code: 0 });
    output.subscribe("session-late", (bytes) => received.push(bytes));

    expect(received).toEqual([]);
    expect(sessions()).toEqual([]);
  });

  it("refuses to retry a session it does not know", async () => {
    const { controller } = setup();

    await expect(controller.retry("session-9", SIZE)).rejects.toMatchObject({
      ipc: { code: "notFound", message: texts.sessions.notFound },
    });
  });

  it("rejects the opening of an unknown connection without adding a session", async () => {
    const { controller } = setup();

    await expect(controller.open({ nodeId: "node-missing", size: SIZE })).rejects.toBeInstanceOf(
      IpcFailure,
    );
    expect(sessions()).toEqual([]);
  });
});
