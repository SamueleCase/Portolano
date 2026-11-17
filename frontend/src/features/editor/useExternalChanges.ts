import { useEffect, useRef, useState } from "react";
import type { UseFormReturn } from "react-hook-form";

import { events, type NodeDetailsView_Serialize } from "../../bindings";
import { reapply } from "./externalChanges";
import { type EditableField, type EditorValues } from "./editorValues";
import { valuesOf } from "./valuesOf";

/** What the editor tells about a document changed by another program. */
export type ExternalNotice =
  { kind: "modified" } | { kind: "reapplied"; conflicts: EditableField[] } | undefined;

/**
 * Keeps the changes of the editor over a node that changes under it
 * (SPEC-UI §4.4). `document-modified-externally` shows a notice; when the
 * detail comes back reloaded, the changes of the user are put back over it
 * (`reapply`) and the fields changed on both sides are reported. A detail
 * read again after the event but equal to the one shown (Query keeps the
 * same object) means a reload that did not touch the node: the notice goes.
 * Returns the node as loaded last, the base of "Salva".
 */
export function useExternalChanges(
  form: UseFormReturn<EditorValues>,
  initial: EditorValues,
  details: NodeDetailsView_Serialize | undefined,
  /** When the detail was last read (`dataUpdatedAt` of its query). */
  detailsUpdatedAt: number,
): { base: EditorValues; notice: ExternalNotice } {
  const [base, setBase] = useState(initial);
  const [notice, setNotice] = useState<ExternalNotice>();
  const loaded = useRef(details);
  /** When the last change from outside was announced, until the detail is read again. */
  const modifiedAt = useRef<number | null>(null);

  const existing = details !== undefined;
  useEffect(() => {
    if (!existing) return undefined;
    const subscription = events.documentModifiedExternally
      .listen(() => {
        modifiedAt.current = Date.now();
        setNotice({ kind: "modified" });
      })
      // Without a backend to listen to (a test without IPC) there is no event.
      .catch(() => null);
    return () =>
      void subscription
        .then((unlisten) => unlisten?.())
        // A backend already gone (the end of a test) has nothing left to unlisten.
        .catch(() => undefined);
  }, [existing]);

  useEffect(() => {
    if (details === undefined || details === loaded.current || loaded.current === undefined) return;
    loaded.current = details;
    const current = form.getValues();
    const result = reapply(base, current, valuesOf(current.kind, details));
    form.reset(result.base);
    for (const field of result.kept) {
      form.setValue(field, current[field], { shouldDirty: true });
    }
    if (Object.keys(current.secrets).length > 0) {
      form.setValue("secrets", current.secrets, { shouldDirty: true });
    }
    setBase(result.base);
    if (modifiedAt.current !== null) {
      modifiedAt.current = null;
      setNotice({ kind: "reapplied", conflicts: result.conflicts });
    }
  }, [details, base, form]);

  // After the effect above: a detail that changed has already ended the wait.
  useEffect(() => {
    if (modifiedAt.current === null || detailsUpdatedAt <= modifiedAt.current) return;
    modifiedAt.current = null;
    setNotice(undefined);
  }, [detailsUpdatedAt]);

  return { base, notice };
}
