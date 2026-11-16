import { beforeEach, describe, expect, it } from "vitest";

import type { AppStatus_Serialize } from "../bindings";
import { createQueryClient } from "../ipc/queryClient";
import { appStatusQuery } from "../ipc/queries";
import { IpcFailure, type IpcError } from "../ipc/result";
import { useToasts } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { initialFakeState } from "../test/fakeBackend";
import { reportFailure } from "./failures";

/** A query client that knows the open document, as after startup. */
function clientWithDocument() {
  const client = createQueryClient();
  const state = initialFakeState();
  const status: AppStatus_Serialize = {
    defaultDocument: { path: state.defaultDocumentPath, exists: false },
    vault: "file",
    document: { name: "Infrastruttura", fileName: "portolano.json", path: "/tmp/portolano.json" },
    lastDocument: null,
    preferences: state.preferences,
  };
  client.setQueryData(appStatusQuery.queryKey, status);
  return client;
}

const failure = (error: IpcError) => new IpcFailure(error);
const toasts = () => useToasts.getState().toasts.map((toast) => toast.text);

describe("the failures of the commands", () => {
  beforeEach(() => {
    useToasts.setState({ toasts: [] });
    useWorkspace.getState().documentSwitched();
  });

  it("shows the banner of a document changed by another program, without a toast", () => {
    reportFailure(
      failure({ code: "modifiedExternally", message: "modificato" }),
      clientWithDocument(),
    );

    expect(useWorkspace.getState().banner).toEqual({
      kind: "modified",
      fileName: "portolano.json",
    });
    expect(toasts()).toEqual([]);
  });

  it("shows the banner of a document file that is gone", () => {
    reportFailure(failure({ code: "documentMissing", message: "mancante" }), clientWithDocument());

    expect(useWorkspace.getState().banner?.kind).toBe("missing");
  });

  it("shows no banner without an open document, but a toast", () => {
    reportFailure(failure({ code: "documentMissing", message: "mancante" }), createQueryClient());

    expect(useWorkspace.getState().banner).toBeNull();
    expect(toasts()).toHaveLength(1);
  });

  it("goes back to the first run when there is no document", async () => {
    const client = clientWithDocument();

    reportFailure(failure({ code: "noDocument", message: "nessun documento" }), client);

    expect(client.getQueryState(appStatusQuery.queryKey)?.isInvalidated).toBe(true);
    expect(toasts()).toEqual([]);
    await client.cancelQueries();
  });

  it("names the candidates of an ambiguous name in the toast", () => {
    reportFailure(
      failure({
        code: "ambiguous",
        message: "nome ambiguo",
        details: {
          kind: "ambiguous",
          candidates: [
            { id: "a", path: "/Clienti/deploy" },
            { id: "b", path: "/Rete/deploy" },
          ],
        },
      }),
      clientWithDocument(),
    );

    expect(toasts()[0]).toContain("Clienti / deploy, Rete / deploy");
  });
});
