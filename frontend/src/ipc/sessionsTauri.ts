import { Channel, invoke, type InvokeArgs } from "@tauri-apps/api/core";

import { IpcFailure, type IpcError } from "./result";
import {
  decodeFrame,
  FrameKind,
  type SessionOpened,
  type SessionState,
  type SessionTransport,
} from "./sessions";

/**
 * The transport of the sessions over the commands and channels of Tauri
 * (SPEC-UI §4.4): output as raw bytes on one channel, states as JSON on another.
 */
export const tauriSessionTransport: SessionTransport = {
  async open(request, handlers) {
    // Handlers go on the channels before the command runs: messages sent
    // before them would be lost.
    const output = new Channel<ArrayBuffer>((buffer) => {
      const frame = decodeFrame(buffer);
      if (frame?.kind === FrameKind.terminalOutput) {
        handlers.onOutput(frame.payload);
      }
    });
    const states = new Channel<SessionState>(handlers.onState);
    return call<SessionOpened>("open_session", { ...request, output, states });
  },
  input: (id, data) => call("session_input", { id, data }),
  resize: (id, size) => call("session_resize", { id, size }),
  close: (id) => call("session_close", { id }),
};

/** Runs `command`; a failure rejects with an {@link IpcFailure}. */
async function call<T = void>(command: string, args: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw new IpcFailure(asIpcError(error));
  }
}

/** The error of a command: the backend rejects with an `IpcError` value. */
function asIpcError(error: unknown): IpcError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    // The shape checked above is the one the backend serializes.
    return error as IpcError;
  }
  return { code: "internal", message: String(error) };
}
