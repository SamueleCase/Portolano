import { EDITABLE_FIELDS, type EditableField, type EditorValues } from "./editorValues";

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

function same(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
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
    if (same(current[field], loaded[field])) continue;
    kept.push(field);
    if (!same(reloaded[field], loaded[field]) && !same(reloaded[field], current[field])) {
      conflicts.push(field);
    }
  }
  return { base: reloaded, kept, conflicts };
}
