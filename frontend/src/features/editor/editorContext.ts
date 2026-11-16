import { createContext, useContext } from "react";

import type { FieldMessage } from "../../components/FormField";
import type { VaultBackend } from "../../components/SecretBadge";
import type { EditorValues } from "./editorValues";
import type { SecretSlots, SlotKey } from "./secretSlots";
import type { ErrorTarget } from "./serverErrors";

/** What the steps of the editor read and change. */
export interface EditorFields {
  values: EditorValues;
  /** Changes a field of the form; its error of the backend goes away. */
  set: <K extends keyof EditorValues>(field: K, value: EditorValues[K]) => void;
  /** The message under `target`: the check of form, or else the backend's. */
  errorOf: (target: ErrorTarget) => string | undefined;
  secrets: SecretSlots;
  /** Opens the dialog of the secret in `key` ("Imposta…", "Sostituisci…"). */
  askSecret: (key: SlotKey) => void;
  /** "Cancella": the removal applies on "Salva". */
  clearSecret: (key: SlotKey) => void;
  backend: VaultBackend;
  /** The node being edited; `undefined` for a new one. */
  nodeId: string | undefined;
}

export const EditorFieldsContext = createContext<EditorFields | null>(null);

/** The message of a field for `TextField` and `TextArea`, from an error text. */
export function errorMessage(text: string | undefined): FieldMessage | undefined {
  return text ? { tone: "error", text } : undefined;
}

export function useEditorFields(): EditorFields {
  const fields = useContext(EditorFieldsContext);
  if (fields === null) {
    // A mistake of the code that mounts the steps, never shown to the user.
    throw new Error("the steps of the editor need an EditorFieldsContext");
  }
  return fields;
}
