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
import {
  ancestorsOf,
  indexTree,
  parentForNew,
  readablePath,
  siblingPlacement,
  type TreeIndex,
} from "../features/tree/treeModel";
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
import { treeQuery, uiPreferencesQuery, vaultStatusQuery } from "../ipc/queries";
import { unwrap } from "../ipc/result";
import { type HostRequest, type NewNodeKind, openRequest } from "../state/requests";
import { showToast } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { reportFailure, runReported, vaultReady } from "./failures";
import { type Feature, useFeatures } from "./features";
import { useWindowServices } from "./windowServices";

/**
 * Everything the window can be asked to do, by the toolbar, the tree and its
 * context menu, the detail, the native menu, the shortcuts and the functions
 * of the hook points alike. The node actions take the node they act on (the
 * selection, or the row of a context menu). A command refused because the
 * vault is locked asks to unlock it and runs again (DEC-041.2); the other
 * failures are shown by `reportFailure`.
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
  /** "Sposta su" / "Sposta giù", Alt+↑ / Alt+↓: one place among its siblings. */
  moveBy: (id: string, step: -1 | 1) => void;
  /**
   * "Imposta…" / "Sostituisci…" of a secret: the password dialog, after the
   * unlock of the vault if it is locked.
   */
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
  /** Closes tab `id`: a session ends with it. */
  closeTab: (id: string) => void;
  setDensity: (density: Density) => void;
  /** The commands running now, for the spinners of their buttons. */
  busy: { duplicate: boolean; remove: boolean };
}

/** The actions of the window; `App` provides them, to the functions of the hook points too. */
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

/** Builds the actions of the window; called once, by `App`. */
export function useWindowActionsValue(): WindowActions {
  const queryClient = useQueryClient();
  const features = useFeatures();
  const { connections, sessions } = useWindowServices();
  const { createNew, openChosen, openRecent } = useDocumentActions();
  const { mutateAsync: duplicateNode, isPending: duplicating } = useDuplicateNode();
  const { mutateAsync: previewDelete, isPending: previewing } = usePreviewDelete();
  const { mutateAsync: editNode } = useEditNode();
  const { mutateAsync: moveNode } = useMoveNode();
  const { mutateAsync: clearSecret } = useClearSecret();
  const { mutateAsync: setPreferences } = useUiPrefsSet();

  const actions = useMemo(() => {
    const offers = (feature: Feature) => features[feature] !== undefined;
    const attempt = (command: () => Promise<unknown>) =>
      void runReported(command, queryClient, offers);
    const tree = (): TreeIndex | undefined => {
      const view = queryClient.getQueryData(treeQuery.queryKey);
      return view === undefined ? undefined : indexTree(view);
    };
    /**
     * A request of the user. With its function missing it still arrives from
     * the native menu, and says why; with a dialog open it is refused.
     */
    const ask = (feature: Feature, request: HostRequest) => {
      if (offers(feature)) openRequest(request);
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
      create: (kind) => {
        const index = tree();
        const parent =
          index === undefined ? null : parentForNew(index, useWorkspace.getState().selectedId);
        ask("editor", { kind: "editor", target: { mode: "add", kind, parent } });
      },
      edit: (id) => ask("editor", { kind: "editor", target: { mode: "edit", id } }),
      duplicate: (id) =>
        attempt(async () => {
          const outcome = await duplicateNode(id);
          // The copy is next to the original, in the same folders.
          const index = tree();
          useWorkspace.getState().reveal(outcome.id, index ? ancestorsOf(index, id) : []);
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
      moveBy: (id, step) => {
        const index = tree();
        const placement = index && siblingPlacement(index, id, step);
        if (!index || !placement) return;
        const destination = index.entries.get(id)?.parentId ?? null;
        attempt(async () => {
          await moveNode({ id, destination, placement });
        });
      },
      requestPassword: (node, field) => {
        if (!offers("password")) {
          showToast({ kind: "info", text: texts.app.reasons.unavailable });
          return;
        }
        void (async () => {
          const vault = await queryClient.fetchQuery({ ...vaultStatusQuery, staleTime: 0 });
          if (vault.locked && !offers(vault.exists ? "unlockVault" : "createFileVault")) {
            // Without the dialog the secret could not be written: say why.
            showToast({ kind: "info", text: texts.app.reasons.vaultLocked });
            return;
          }
          if (await vaultReady(queryClient, offers)) {
            openRequest({ kind: "password", node, field });
          }
        })().catch((failure: unknown) => reportFailure(failure, queryClient));
      },
      clearSecret: (id, slot) =>
        attempt(async () => {
          await clearSecret({ id, slot });
          showToast({ kind: "success", text: texts.app.toasts.secretCleared });
        }),
      openImport: () => ask("import", { kind: "import" }),
      openValidation: () => {
        if (offers("validation")) useWorkspace.getState().openValidation();
        else showToast({ kind: "info", text: texts.app.reasons.unavailable });
      },
      openPreferences: () => ask("preferences", { kind: "preferences" }),
      unlockVault: () => ask("unlockVault", { kind: "unlockVault" }),
      createFileVault: () => ask("createFileVault", { kind: "createFileVault" }),
      newDocument: () => void createNew(),
      openDocument: () => void openChosen(),
      openRecent: (path) => void openRecent(path),
      closeTab: (id) => {
        useWorkspace.getState().tabClosed(id);
        if (useSessionsStore.getState().sessions.some((session) => session.id === id)) {
          attempt(() => sessions.controller.close(id));
        }
      },
      setDensity: (density) =>
        attempt(async () => {
          const current = queryClient.getQueryData(uiPreferencesQuery.queryKey);
          await setPreferences({ ...current, density });
        }),
    } satisfies Omit<WindowActions, "busy">;
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
  ]);

  return useMemo(
    () => ({ ...actions, busy: { duplicate: duplicating, remove: previewing } }),
    [actions, duplicating, previewing],
  );
}
