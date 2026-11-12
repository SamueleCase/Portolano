import type { IpcError_Serialize } from "../bindings";

/** The error of a command, as the backend sends it (SPEC-UI §4.2). */
export type IpcError = IpcError_Serialize;

/** The result of a generated command: errors are values, never thrown. */
export type IpcResult<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

/** A failed command, thrown where a promise has to reject (TanStack Query). */
export class IpcFailure extends Error {
  readonly ipc: IpcError;

  constructor(ipc: IpcError) {
    super(ipc.message);
    this.name = "IpcFailure";
    this.ipc = ipc;
  }
}

/** The data of `result`, or an {@link IpcFailure} with its error. */
export function unwrap<T>(result: IpcResult<T>): T {
  if (result.status === "error") {
    throw new IpcFailure(result.error);
  }
  return result.data;
}
