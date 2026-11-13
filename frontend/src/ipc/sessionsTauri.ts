import { Channel } from "@tauri-apps/api/core";

import { commands, type SessionState_Deserialize } from "../bindings";
import { unwrap } from "./result";
import { decodeFrame, FrameKind, type SessionState, type SessionTransport } from "./sessions";
import { serializedTransport } from "./sessionsQueue";

/**
 * The transport of the sessions over the generated commands and two channels
 * (SPEC-UI §4.4): output as raw bytes, states as JSON; input and sizes go one
 * call at a time per session, in order.
 */
export const tauriSessionTransport: SessionTransport = serializedTransport({
  async open({ nodeId, size, options, login }, handlers) {
    // Handlers go on the channels before the command runs: messages sent
    // before them would be lost.
    const output = new Channel<ArrayBuffer>((buffer) => {
      const frame = decodeFrame(buffer);
      if (frame?.kind === FrameKind.terminalOutput) {
        handlers.onOutput(frame.payload);
      }
    });
    const states = new Channel<SessionState_Deserialize>((state) =>
      handlers.onState(asSent(state)),
    );
    return unwrap(
      await commands.openSession(nodeId, options ?? {}, login ?? null, size, output, states),
    );
  },
  async input(id, data) {
    unwrap(await commands.sessionInput(id, data));
  },
  async resize(id, size) {
    unwrap(await commands.sessionResize(id, size));
  },
  async close(id) {
    unwrap(await commands.sessionClose(id));
  },
});

/**
 * A state as the backend sends it. The bindings type a channel with the shape
 * the backend would read, where the code of a failure may be absent; the
 * backend always writes it, `null` when there is none.
 */
function asSent(state: SessionState_Deserialize): SessionState {
  return state.state === "failed" ? { ...state, code: state.code ?? null } : state;
}
