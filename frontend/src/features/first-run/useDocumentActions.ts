import { useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import { reportFailure } from "../../app/failures";
import {
  useChooseDocument,
  useChooseNewDocument,
  useCreateDocument,
  useOpenDocument,
  useReloadDocument,
} from "../../ipc/mutations";
import { useWorkspace } from "../../state/workspace";

/** What the document commands are doing, for the spinners of their buttons. */
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
  /** "Ricrea vuoto": an empty document named `name` where the missing file was. */
  recreate: (path: string, name: string) => Promise<void>;
  busy: DocumentBusy;
}

/** The name inside a new document: its file name without the extension. */
export function documentName(fileName: string): string {
  return fileName.replace(/\.json$/i, "") || fileName;
}

/** The document commands, with their failures shown (`reportFailure`). */
export function useDocumentActions(): DocumentActions {
  const queryClient = useQueryClient();
  const chooseDocument = useChooseDocument();
  const chooseNewDocument = useChooseNewDocument();
  const openDocument = useOpenDocument();
  const createDocument = useCreateDocument();
  const reloadDocument = useReloadDocument();

  const busy: DocumentBusy = chooseNewDocument.isPending
    ? "new"
    : chooseDocument.isPending
      ? "open"
      : reloadDocument.isPending
        ? "reload"
        : openDocument.isPending
          ? "recent"
          : createDocument.isPending
            ? "recreate"
            : undefined;

  const { mutateAsync: chooseOpen } = chooseDocument;
  const { mutateAsync: chooseNew } = chooseNewDocument;
  const { mutateAsync: open } = openDocument;
  const { mutateAsync: create } = createDocument;
  const { mutateAsync: reloadNow } = reloadDocument;

  const actions = useMemo(() => {
    const attempt = async (action: () => Promise<unknown>) => {
      try {
        await action();
      } catch (failure) {
        reportFailure(failure, queryClient);
      }
    };
    const switched = () => useWorkspace.getState().documentSwitched();
    return {
      createNew: () =>
        attempt(async () => {
          const file = await chooseNew();
          if (file === null) return;
          await create({ source: { kind: "chosen" }, name: documentName(file.fileName) });
          switched();
        }),
      openChosen: () =>
        attempt(async () => {
          const file = await chooseOpen();
          if (file === null) return;
          await open({ kind: "chosen" });
          switched();
        }),
      openRecent: (path: string) =>
        attempt(async () => {
          await open({ kind: "recent", path });
          switched();
        }),
      reload: () =>
        attempt(async () => {
          await reloadNow();
          useWorkspace.getState().setBanner(null);
        }),
      recreate: (path: string, name: string) =>
        attempt(async () => {
          await create({ source: { kind: "recent", path }, name });
          switched();
        }),
    };
  }, [queryClient, chooseNew, chooseOpen, open, create, reloadNow]);

  return { ...actions, busy };
}
