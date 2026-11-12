import { Channel, type InvokeArgs } from "@tauri-apps/api/core";
import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it, vi } from "vitest";

import { rejectWith } from "../test/fakeBackend";
import { IpcFailure } from "./result";
import { FRAME_VERSION, FrameKind, type SessionHandlers, type SessionState } from "./sessions";
import { tauriSessionTransport } from "./sessionsTauri";

/** The channel passed to a command as `name`. */
function channel(args: InvokeArgs | undefined, name: string): Channel<unknown> {
  const value = (args as Record<string, unknown> | undefined)?.[name];
  if (!(value instanceof Channel)) {
    throw new Error(`no channel "${name}"`);
  }
  return value;
}

function handlers(): SessionHandlers & { outputs: string[]; states: SessionState[] } {
  const outputs: string[] = [];
  const states: SessionState[] = [];
  return {
    outputs,
    states,
    onOutput: (bytes) => outputs.push(new TextDecoder().decode(bytes)),
    onState: (state) => states.push(state),
  };
}

describe("tauriSessionTransport", () => {
  it("opens a session and passes on its output frames and states", async () => {
    mockIPC((command, args) => {
      expect(command).toBe("open_session");
      expect(args).toMatchObject({ nodeId: "node-1", size: { cols: 80, rows: 24 } });
      channel(args, "states").onmessage({ state: "connected" });
      const frame = new Uint8Array([FRAME_VERSION, FrameKind.terminalOutput, 111, 107]);
      channel(args, "output").onmessage(frame.buffer);
      return { id: "session-1", kind: "terminal" };
    });
    const received = handlers();

    const opened = await tauriSessionTransport.open(
      { nodeId: "node-1", size: { cols: 80, rows: 24 } },
      received,
    );

    expect(opened).toEqual({ id: "session-1", kind: "terminal" });
    expect(received.states).toEqual([{ state: "connected" }]);
    expect(received.outputs).toEqual(["ok"]);
  });

  it("drops frames it cannot read", async () => {
    mockIPC((_command, args) => {
      channel(args, "output").onmessage(new Uint8Array([FRAME_VERSION + 1, 0, 120]).buffer);
      return { id: "session-1", kind: "terminal" };
    });
    const received = handlers();

    await tauriSessionTransport.open({ nodeId: "node-1", size: { cols: 80, rows: 24 } }, received);

    expect(received.outputs).toEqual([]);
  });

  it("sends input, size and closing to the session", async () => {
    const calls = vi.fn();
    mockIPC((command, args) => {
      calls(command, args);
      return null;
    });

    await tauriSessionTransport.input("session-1", "ls\r");
    await tauriSessionTransport.resize("session-1", { cols: 100, rows: 30 });
    await tauriSessionTransport.close("session-1");

    expect(calls.mock.calls).toEqual([
      ["session_input", { id: "session-1", data: "ls\r" }],
      ["session_resize", { id: "session-1", size: { cols: 100, rows: 30 } }],
      ["session_close", { id: "session-1" }],
    ]);
  });

  it("rejects with the error of the backend", async () => {
    mockIPC(() => rejectWith({ code: "notFound", message: "sessione inesistente" }));

    const result = tauriSessionTransport.close("session-1");

    await expect(result).rejects.toBeInstanceOf(IpcFailure);
    await expect(result).rejects.toMatchObject({ ipc: { code: "notFound" } });
  });
});
