import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import type { UseFormReturn } from "react-hook-form";

import { events, type NodeDetailsView_Serialize } from "../../bindings";
import { reloadDocumentKey } from "../../ipc/mutations";
import { nodeDetailsQuery } from "../../ipc/queries";
import { reapply } from "./externalChanges";
import { type EditableField, type EditorValues, sameValue } from "./editorValues";
import { valuesOf } from "./valuesOf";

/** What the editor tells about a document changed by another program. */
export type ExternalNotice =
  { kind: "modified" } | { kind: "reapplied"; conflicts: EditableField[] } | undefined;

/** What happened to the node between the change from outside and "Ricarica". */
interface Pending {
  /** "Ricarica" succeeded; the notice ends once its detail has been put back. */
  reloaded: boolean;
  /** Whether a detail read since then changed a field of the form. */
  changed: boolean;
  /** The fields changed on both sides. */
  conflicts: Set<EditableField>;
}

/**
 * Keeps the changes of the editor over a node that changes under it
 * (SPEC-UI §4.4). `document-modified-externally` shows a notice that ends
 * only when "Ricarica" succeeds (DEC-043.7), not at any reading of the detail
 * (a locked vault, the window coming back): then, if the node changed, the
 * notice says so with the fields changed on both sides, and else it goes.
 * Every new detail has the changes of the user put back over it
 * (`reapply`). Returns the node as loaded last, the base of "Salva".
 */
export function useExternalChanges(
  form: UseFormReturn<EditorValues>,
  initial: EditorValues,
  details: NodeDetailsView_Serialize | undefined,
): { base: EditorValues; notice: ExternalNotice } {
  const queryClient = useQueryClient();
  const [base, setBase] = useState(initial);
  const [notice, setNotice] = useState<ExternalNotice>();
  const loaded = useRef(details);
  const pending = useRef<Pending | null>(null);

  /** Ends the wait for "Ricarica" with what the reloaded node brought. */
  const settle = (done: Pending) => {
    pending.current = null;
    setNotice(done.changed ? { kind: "reapplied", conflicts: [...done.conflicts] } : undefined);
  };

  const id = details?.id;
  useEffect(() => {
    if (id === undefined) return undefined;
    const subscription = events.documentModifiedExternally
      .listen(() => {
        pending.current = { reloaded: false, changed: false, conflicts: new Set() };
        setNotice({ kind: "modified" });
      })
      // Without a backend to listen to (a test without IPC) there is no event.
      .catch(() => null);
    const stopWatching = queryClient.getMutationCache().subscribe((event) => {
      const waiting = pending.current;
      if (
        waiting === null ||
        event.type !== "updated" ||
        event.action.type !== "success" ||
        !sameValue(event.mutation.options.mutationKey, reloadDocumentKey)
      ) {
        return;
      }
      waiting.reloaded = true;
      // The detail read by "Ricarica" is already put back, or was the same
      // object as before (Query keeps it when nothing changed); otherwise
      // the next render brings it and the effect below settles.
      if (queryClient.getQueryData(nodeDetailsQuery(id).queryKey) === loaded.current) {
        settle(waiting);
      }
    });
    return () => {
      stopWatching();
      void subscription
        .then((unlisten) => unlisten?.())
        // A backend already gone (the end of a test) has nothing left to unlisten.
        .catch(() => undefined);
    };
  }, [id, queryClient]);

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
    const waiting = pending.current;
    if (waiting === null) return;
    waiting.changed ||= result.changed;
    for (const field of result.conflicts) waiting.conflicts.add(field);
    if (waiting.reloaded) settle(waiting);
  }, [details, base, form]);

  return { base, notice };
}
