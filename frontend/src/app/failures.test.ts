import { afterEach, beforeEach, describe, expect, it } from "vitest";

import type { AppStatus_Serialize } from "../bindings";
import { createQueryClient } from "../ipc/queryClient";
import { appStatusQuery } from "../ipc/queries";
import { IpcFailure, type IpcError } from "../ipc/result";
import { answerRequest, closeAllRequests, useRequests } from "../state/requests";
import { useToasts } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { initialFakeState, installFakeBackend } from "../test/fakeBackend";
import { reportFailure, runReported } from "./failures";

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

describe("a command refused with the vault locked", () => {
  const locked = () => Promise.reject(failure({ code: "vaultLocked", message: "vault bloccato" }));
  let stopAnswering: () => void = () => undefined;

  /** The vault dialog answers `done` as soon as it opens. */
  function vaultDialogAnswers(done: boolean) {
    stopAnswering = useRequests.subscribe(({ stack }) => {
      const open = stack.at(-1);
      if (open) answerRequest(open.id, done);
    });
  }

  beforeEach(() => {
    useToasts.setState({ toasts: [] });
    installFakeBackend();
  });

  afterEach(() => {
    stopAnswering();
    closeAllRequests();
  });

  it("runs once more after the unlock, and only once", async () => {
    vaultDialogAnswers(true);
    let runs = 0;

    await runReported(
      () => {
        runs += 1;
        return locked();
      },
      createQueryClient(),
      () => true,
    );

    expect(runs).toBe(2);
    expect(toasts()).toHaveLength(1);
  });

  it("does not run again when the unlock is cancelled", async () => {
    vaultDialogAnswers(false);
    let runs = 0;

    await runReported(
      () => {
        runs += 1;
        return locked();
      },
      createQueryClient(),
      () => true,
    );

    expect(runs).toBe(1);
    expect(toasts()).toHaveLength(1);
  });

  it("says so in a toast without the unlock dialog, asking nothing", async () => {
    let asked = false;
    stopAnswering = useRequests.subscribe(() => {
      asked = true;
    });

    await runReported(locked, createQueryClient(), () => false);

    expect(asked).toBe(false);
    expect(toasts()).toHaveLength(1);
  });
});
