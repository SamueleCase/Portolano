import { EDITABLE_FIELDS, type EditableField, type EditorValues, sameValue } from "./editorValues";

/**
 * The changes of the editor kept as a patch over a document changed by
 * another program (SPEC-UI §4.4): after "Ricarica", the fields the user did
 * not touch take the new values; those the user changed keep the user's
 * value, and the ones changed on both sides, differently, are reported.
 */
export interface Reapplied {
  /** The node as reloaded: what "Salva" compares with from now on. */
  base: EditorValues;
  /** The fields that keep the value of the user. */
  kept: EditableField[];
  /** The fields changed by both, to warn about. */
  conflicts: EditableField[];
}

/** Puts the changes from `loaded` to `current` over `reloaded`. */
export function reapply(
  loaded: EditorValues,
  current: EditorValues,
  reloaded: EditorValues,
): Reapplied {
  const kept: EditableField[] = [];
  const conflicts: EditableField[] = [];
  for (const field of EDITABLE_FIELDS) {
    if (sameValue(current[field], loaded[field])) continue;
    kept.push(field);
    if (!sameValue(reloaded[field], loaded[field]) && !sameValue(reloaded[field], current[field])) {
      conflicts.push(field);
    }
  }
  return { base: reloaded, kept, conflicts };
}
