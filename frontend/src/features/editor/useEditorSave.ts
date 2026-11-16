import { useState } from "react";

import type { NodeDetailsView_Serialize } from "../../bindings";
import { presentFailure } from "../../ipc/errors";
import { useAddNode, useEditNode } from "../../ipc/mutations";
import { requestVault } from "../../state/requests";
import { editInput, newNodeInput } from "./editorRequest";
import type { EditorValues } from "./editorValues";
import type { SlotKey } from "./secretSlots";
import { type RoutedError, routeError } from "./serverErrors";
import type { StagedSecrets } from "./stagedSecrets";

/** The node saved: where it is now. */
export interface SavedNode {
  id: string;
  path: string;
}

export type SaveResult =
  | { status: "saved"; node: SavedNode }
  /** The user did not unlock the vault: nothing was sent, the staged secrets stay. */
  | { status: "cancelled" }
  /** The backend refused: the staged secrets are gone with the request. */
  | { status: "failed"; error: RoutedError; secretsLost: boolean };

/** What "Salva" works on: the form, the node as loaded, and its detail. */
export interface SaveRequest {
  values: EditorValues;
  /** The form as loaded; for a new node, the blank form. */
  base: EditorValues;
  /** The detail of the node being edited; `undefined` for a new one. */
  details: NodeDetailsView_Serialize | undefined;
  /** Whether the saved node has a secret in a slot. */
  stored: (key: SlotKey) => boolean;
}

/** Whether saving writes or removes secrets, and so needs the file vault open. */
function touchesSecrets({ values, base }: SaveRequest): boolean {
  const kept = new Set(values.properties.map((row) => row.saved));
  return (
    Object.keys(values.secrets).length > 0 ||
    base.properties.some((row) => row.saved !== null && !kept.has(row.saved)) ||
    (base.keyMode === "vault" && values.keyMode !== "vault")
  );
}

/**
 * "Salva" of the editor: the vault is opened first if the secrets need it,
 * then the staged secrets leave the editor in the one command that saves
 * them (`add_node` or `edit_node`, a move included), whatever its outcome
 * (DEC-038.2, DEC-040.2).
 */
export function useEditorSave(staged: StagedSecrets): {
  save: (request: SaveRequest) => Promise<SaveResult>;
  saving: boolean;
} {
  const addNode = useAddNode();
  const editNode = useEditNode();
  const [saving, setSaving] = useState(false);

  const send = async ({ values, base, details, stored }: SaveRequest): Promise<SavedNode> => {
    const secrets = staged.take();
    if (details === undefined) {
      const added = await addNode.mutateAsync(newNodeInput(values, secrets));
      return { id: added.id, path: added.path };
    }
    const edited = await editNode.mutateAsync({
      id: details.id,
      edit: editInput(values, base, details.connection?.settings, secrets, stored),
    });
    return { id: details.id, path: edited.path };
  };

  const save = async (request: SaveRequest): Promise<SaveResult> => {
    const failed = (failure: unknown, secretsLost: boolean): SaveResult => ({
      status: "failed",
      error: routeError(presentFailure(failure), request.values.kind),
      secretsLost,
    });
    setSaving(true);
    try {
      try {
        // The unlock of the window, on top of the editor (DEC-041.1).
        if (touchesSecrets(request) && !(await requestVault())) {
          return { status: "cancelled" };
        }
      } catch (failure) {
        // The state of the vault could not be read: nothing was sent.
        return failed(failure, false);
      }
      const staging = Object.values(request.values.secrets).includes("set");
      try {
        return { status: "saved", node: await send(request) };
      } catch (failure) {
        return failed(failure, staging);
      }
    } finally {
      setSaving(false);
    }
  };

  return { save, saving };
}
