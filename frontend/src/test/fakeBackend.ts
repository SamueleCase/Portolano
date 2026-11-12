import { mockIPC } from "@tauri-apps/api/mocks";

import type { AppStatus } from "../bindings";
import type { IpcError } from "../ipc/result";

/** What the fake backend answers; invented data only (SPEC §1.3). */
export interface FakeBackendState {
  appStatus: AppStatus;
}

/** The state the end-to-end tests start from. */
export function initialFakeState(): FakeBackendState {
  return {
    appStatus: {
      defaultDocument: {
        path: "/Users/utente-di-prova/Library/Application Support/dev.portolano.Portolano/portolano.json",
        exists: false,
      },
      vault: "keychain",
    },
  };
}

/**
 * Answers the commands of the generated bindings from `state`, in memory, in
 * place of the IPC of Tauri: for the end-to-end tests and the component
 * tests.
 */
export function installFakeBackend(state: FakeBackendState = initialFakeState()): void {
  mockIPC((command) => {
    switch (command) {
      case "app_status":
        return state.appStatus;
      default:
        return rejectWith({ code: "internal", message: `comando non simulato: ${command}` });
    }
  });
}

/** A failed command: the backend rejects with the error value itself. */
export function rejectWith(error: IpcError): Promise<never> {
  // Commands reject with an `IpcError` object, not an `Error`: the generated
  // bindings tell the two apart to return command errors as values.
  // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
  return Promise.reject(error);
}
