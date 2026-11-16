import { useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, useMemo } from "react";

import {
  commands,
  type Density,
  type NodeRef,
  type PlacementInput,
  type SecretSlotInput,
} from "../bindings";
import { useDeleteConfirm } from "../features/details/deleteStore";
import { useDocumentActions } from "../features/first-run/useDocumentActions";
import { useSessionsStore } from "../features/sessions/store";
import { indexTree, parentForNew, readablePath, type TreeIndex } from "../features/tree/treeModel";
import { interpolate, plural } from "../i18n/format";
import { texts } from "../i18n/it";
import {
  useClearSecret,
  useDuplicateNode,
  useEditNode,
  useMoveNode,
  usePreviewDelete,
  useUiPrefsSet,
} from "../ipc/mutations";
import { appStatusQuery, treeQuery, uiPreferencesQuery } from "../ipc/queries";
import { unwrap } from "../ipc/result";
import { type NewNodeKind, openRequest } from "../state/requests";
import { showToast } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { reportFailure } from "./failures";
import { type Feature, useFeatures } from "./features";
import { useWindowServices } from "./windowServices";

/**
 * Everything the window can be asked to do, by the toolbar, the tree and its
 * context menu, the detail, the native menu and the shortcuts alike. The node
 * actions take the node they act on (the selection, or the row of a context
 * menu); failures are shown by `reportFailure`.
 */
export interface WindowActions {
  /** Whether the function of a hook point is mounted (`features.ts`). */
  offers: (feature: Feature) => boolean;
  connect: (id: string) => void;
  openInTerminal: (id: string) => void;
  /** "+ Connessione" (SSH or RDP), "+ Cartella", "+ Credenziale" (ambiguity 10). */
  create: (kind: NewNodeKind) => void;
  edit: (id: string) => void;
  duplicate: (id: string) => void;
  /** Asks what deleting `id` would remove, then the confirmation. */
  remove: (id: string) => void;
  toggleFavorite: (id: string) => void;
  move: (id: string, destination: string | null, placement: PlacementInput) => void;
  requestPassword: (node: NodeRef, field: SecretSlotInput) => void;
  clearSecret: (id: string, slot: SecretSlotInput) => void;
  openImport: () => void;
  openValidation: () => void;
  openPreferences: () => void;
  unlockVault: () => void;
  createFileVault: () => void;
  newDocument: () => void;
  openDocument: () => void;
  openRecent: (path: string) => void;
  reload: () => void;
  recreate: () => void;
  /** Closes tab `id`: a session ends with it. */
  closeTab: (id: string) => void;
  setDensity: (density: Density) => void;
}

/** The actions of the window; `MainWindow` provides them. */
export const WindowActionsContext = createContext<WindowActions | null>(null);

/** The actions of the window. */
export function useWindowActions(): WindowActions {
  const actions = useContext(WindowActionsContext);
  if (actions === null) {
    // A mistake of the code that mounts the window, never shown to the user.
    throw new Error("the window actions need a WindowActionsContext");
  }
  return actions;
}

/** Builds the actions of the window; called once, by `MainWindow`. */
export function useWindowActionsValue(): WindowActions {
  const queryClient = useQueryClient();
  const features = useFeatures();
  const { connections, sessions } = useWindowServices();
  const documents = useDocumentActions();
  const { mutateAsync: duplicateNode } = useDuplicateNode();
  const { mutateAsync: previewDelete } = usePreviewDelete();
  const { mutateAsync: editNode } = useEditNode();
  const { mutateAsync: moveNode } = useMoveNode();
  const { mutateAsync: clearSecret } = useClearSecret();
  const { mutateAsync: setPreferences } = useUiPrefsSet();
  const { createNew, openChosen, openRecent, reload, recreate } = documents;

  return useMemo(() => {
    const report = (failure: unknown) => reportFailure(failure, queryClient);
    const attempt = (action: () => Promise<unknown>) => void action().catch(report);
    const tree = (): TreeIndex | undefined => {
      const view = queryClient.getQueryData(treeQuery.queryKey);
      return view === undefined ? undefined : indexTree(view);
    };
    const offers = (feature: Feature) => features[feature] !== undefined;
    /** A request whose function is missing still arrives from the native menu. */
    const ifOffered = (feature: Feature, action: () => void) => {
      if (offers(feature)) action();
      else showToast({ kind: "info", text: texts.app.reasons.unavailable });
    };

    return {
      offers,
      connect: (id) => void connections.connect(id),
      openInTerminal: (id) =>
        attempt(async () => {
          unwrap(await commands.openInTerminal(id));
          showToast({ kind: "success", text: texts.app.toasts.openedInTerminal });
        }),
      create: (kind) =>
        ifOffered("editor", () => {
          const index = tree();
          const parent =
            index === undefined ? null : parentForNew(index, useWorkspace.getState().selectedId);
          openRequest({ kind: "editor", target: { mode: "add", kind, parent } });
        }),
      edit: (id) =>
        ifOffered("editor", () => openRequest({ kind: "editor", target: { mode: "edit", id } })),
      duplicate: (id) =>
        attempt(async () => {
          const outcome = await duplicateNode(id);
          useWorkspace.getState().select(outcome.id);
          showToast({
            kind: "success",
            text: interpolate(texts.app.toasts.duplicated, { path: readablePath(outcome.path) }),
          });
          if (outcome.renamed.length > 0) {
            showToast({
              kind: "info",
              text: interpolate(plural(outcome.renamed.length, texts.app.toasts.renamed), {
                paths: outcome.renamed.map((node) => readablePath(node.path)).join(", "),
              }),
            });
          }
        }),
      remove: (id) =>
        attempt(async () => {
          const preview = await previewDelete(id);
          const name = tree()?.entries.get(id)?.node.name ?? readablePath(preview.path);
          useDeleteConfirm.getState().ask(preview, name);
        }),
      toggleFavorite: (id) =>
        attempt(async () => {
          const favorite = tree()?.entries.get(id)?.node.favorite ?? false;
          await editNode({ id, edit: { details: { favorite: !favorite } } });
        }),
      move: (id, destination, placement) =>
        attempt(async () => {
          await moveNode({ id, destination, placement });
        }),
      requestPassword: (node, field) =>
        ifOffered("password", () => openRequest({ kind: "password", node, field })),
      clearSecret: (id, slot) =>
        attempt(async () => {
          await clearSecret({ id, slot });
          showToast({ kind: "success", text: texts.app.toasts.secretCleared });
        }),
      openImport: () => ifOffered("import", () => openRequest({ kind: "import" })),
      openValidation: () => ifOffered("validation", () => useWorkspace.getState().openValidation()),
      openPreferences: () => ifOffered("preferences", () => openRequest({ kind: "preferences" })),
      unlockVault: () => ifOffered("unlockVault", () => openRequest({ kind: "unlockVault" })),
      createFileVault: () =>
        ifOffered("createFileVault", () => openRequest({ kind: "createFileVault" })),
      newDocument: () => void createNew(),
      openDocument: () => void openChosen(),
      openRecent: (path) => void openRecent(path),
      reload: () => void reload(),
      recreate: () => {
        const document = queryClient.getQueryData(appStatusQuery.queryKey)?.document;
        if (document) void recreate(document.path, document.name);
      },
      closeTab: (id) => {
        useWorkspace.getState().tabClosed(id);
        if (useSessionsStore.getState().sessions.some((session) => session.id === id)) {
          void sessions.controller.close(id).catch(report);
        }
      },
      setDensity: (density) =>
        attempt(async () => {
          const current = queryClient.getQueryData(uiPreferencesQuery.queryKey);
          await setPreferences({ ...current, density });
        }),
    } satisfies WindowActions;
  }, [
    queryClient,
    features,
    connections,
    sessions,
    duplicateNode,
    previewDelete,
    editNode,
    moveNode,
    clearSecret,
    setPreferences,
    createNew,
    openChosen,
    openRecent,
    reload,
    recreate,
  ]);
}
