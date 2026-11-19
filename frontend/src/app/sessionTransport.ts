import { commands } from "../bindings";
import type { SessionTransport } from "../ipc/sessions";
import { tauriSessionTransport } from "../ipc/sessionsTauri";
import { IpcFailure, unwrap } from "../ipc/result";

/**
 * The transport of the sessions of the app: the commands and channels of
 * Tauri. The end-to-end tests run the frontend alone, over the fake backend
 * (`main.tsx`), and need the fake transport, where every connection of the
 * fake document is a terminal that works. Vite drops this branch, and the
 * fake with it, from the other builds.
 */
export const appSessionTransport: SessionTransport =
  import.meta.env.MODE === "e2e" ? await fakeTransport() : tauriSessionTransport;

async function fakeTransport(): Promise<SessionTransport> {
  const { FakeSessionTransport, fakeScenarios, recordTraffic, workingConnection } =
    await import("../test/fakeSessions");
  const transport = new FakeSessionTransport(fakeScenarios, workingConnection);
  return {
    // As `open_session` does (DEC-042): a connection whose credential has
    // secrets fails with the vault locked; one without secrets opens.
    open: async (request, handlers) => {
      const details = unwrap(await commands.nodeDetails(request.nodeId));
      if (details.vaultLocked) {
        const error = {
          code: "vaultLocked" as const,
          message: "il vault è bloccato: sbloccalo con la master password",
        };
        // In the record of the sentinel e2e, as the other answers of the fake.
        recordTraffic(`error open_session: ${JSON.stringify(error)}`);
        throw new IpcFailure(error);
      }
      return transport.open(request, handlers);
    },
    input: (id, data) => transport.input(id, data),
    resize: (id, size) => transport.resize(id, size),
    close: (id) => transport.close(id),
  };
}
