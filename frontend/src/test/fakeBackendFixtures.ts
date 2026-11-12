import { commands } from "../bindings";
import type { IpcResult } from "../ipc/result";
import { type FakeBackendState, initialFakeState, installFakeBackend } from "./fakeBackend";

/** Helpers of the tests of the fake backend and of the screens built on it. */

/** The data of a successful command; a failure fails the test with its message. */
export function ok<T>(result: IpcResult<T>): T {
  if (result.status === "error") throw new Error(result.error.message);
  return result.data;
}

/** The error of a failed command; a success fails the test. */
export function refused<T>(result: IpcResult<T>) {
  if (result.status === "ok") throw new Error("il comando doveva essere rifiutato");
  return result.error;
}

/** The id of the first node named `name`, in tree order. */
export async function idOf(name: string): Promise<string> {
  const queue = [...ok(await commands.tree()).nodes];
  for (let node = queue.shift(); node; node = queue.shift()) {
    if (node.name === name) return node.id;
    queue.push(...node.children);
  }
  throw new Error(`nodo ${name} assente`);
}

/** The fake backend with the sample document and the vault unlocked. */
export function unlocked(): FakeBackendState {
  const state = initialFakeState();
  state.vault.locked = false;
  return installFakeBackend(state);
}
