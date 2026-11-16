import { useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";

import { reportFailure } from "../../app/failures";
import { commands } from "../../bindings";
import { queryKeys } from "../../ipc/keys";
import { useCommandMutation } from "../../ipc/mutation";
import {
  useChooseDocument,
  useChooseNewDocument,
  useCreateDocument,
  useOpenDocument,
  useReloadDocument,
} from "../../ipc/mutations";
import { closeAllRequests } from "../../state/requests";
import { useWorkspace } from "../../state/workspace";

/** The document command running now, for the spinner of its button. */
export type DocumentBusy = "new" | "open" | "recent" | "reload" | "recreate" | undefined;

/** Opening, creating and reloading the document (SPEC-UI §4.3 "Documento"). */
export interface DocumentActions {
  /** "Crea un nuovo documento": the native dialog, then the new document. */
  createNew: () => Promise<void>;
  /** "Apri un documento esistente" and "Apri…": the native dialog, then the document. */
  openChosen: () => Promise<void>;
  openRecent: (path: string) => Promise<void>;
  /** "Ricarica": the document as the file has it now. */
  reload: () => Promise<void>;
  /** "Ricrea": the document open in memory written where its missing file was. */
  recreate: () => Promise<void>;
  busy: DocumentBusy;
}

/** The name inside a new document: its file name without the extension. */
export function documentName(fileName: string): string {
  return fileName.replace(/\.json$/i, "") || fileName;
}

/** The document commands, with their failures shown (`reportFailure`). */
export function useDocumentActions(): DocumentActions {
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState<DocumentBusy>();
  const { mutateAsync: chooseOpen } = useChooseDocument();
  const { mutateAsync: chooseNew } = useChooseNewDocument();
  const { mutateAsync: open } = useOpenDocument();
  const { mutateAsync: create } = useCreateDocument();
  const { mutateAsync: reloadNow } = useReloadDocument();
  const { mutateAsync: recreateNow } = useCommandMutation(
    () => commands.recreateDocument(),
    [queryKeys.appStatus, queryKeys.document],
  );

  const actions = useMemo(() => {
    const attempt = async (name: DocumentBusy, action: () => Promise<unknown>) => {
      setBusy(name);
      try {
        await action();
      } catch (failure) {
        reportFailure(failure, queryClient);
      } finally {
        setBusy(undefined);
      }
    };
    const switched = () => {
      // Nothing open on the previous document stays open on the new one.
      closeAllRequests();
      useWorkspace.getState().documentSwitched();
    };
    const banner = () => useWorkspace.getState().setBanner(null);
    return {
      createNew: () =>
        attempt("new", async () => {
          const file = await chooseNew();
          if (file === null) return;
          await create({ source: { kind: "chosen" }, name: documentName(file.fileName) });
          switched();
        }),
      openChosen: () =>
        attempt("open", async () => {
          const file = await chooseOpen();
          if (file === null) return;
          await open({ kind: "chosen" });
          switched();
        }),
      openRecent: (path: string) =>
        attempt("recent", async () => {
          await open({ kind: "recent", path });
          switched();
        }),
      reload: () =>
        attempt("reload", async () => {
          await reloadNow();
          banner();
        }),
      recreate: () =>
        attempt("recreate", async () => {
          await recreateNow();
          banner();
        }),
    };
  }, [queryClient, chooseNew, chooseOpen, open, create, reloadNow, recreateNow]);

  return { ...actions, busy };
}
