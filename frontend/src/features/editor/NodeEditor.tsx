import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "../../components/Button";
import { SidePanel } from "../../components/SidePanel";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { nodeDetailsQuery } from "../../ipc/queries";
import styles from "./Editor.module.css";
import { EditorForm } from "./EditorForm";
import { blankValues, type EditorKind } from "./editorValues";
import { editorKindOf, valuesOf } from "./valuesOf";
import type { SavedNode } from "./useEditorSave";

/**
 * What opens the editor (the `openEditor` of the frame, DEC-035): a new node
 * of `kind` in folder `parent` (`null` is the root; ambiguity 10: the
 * selection of the main window, changeable at step 1), or node `id`.
 */
export type EditorRequest =
  { mode: "add"; kind: EditorKind; parent: string | null } | { mode: "edit"; id: string };

interface NodeEditorProps {
  /** The node to edit; `null` closes the editor. */
  request: EditorRequest | null;
  /** "Annulla", Esc or a click on the veil: nothing is saved, the staged secrets go. */
  onClose: () => void;
  /** "Salva" succeeded: the node and its path now. The host closes the editor. */
  onSaved: (node: SavedNode) => void;
}

/** The editor of a node in a side panel (inventario §4.17, §5.5). */
export function NodeEditor({ request, onClose, onSaved }: NodeEditorProps) {
  if (request === null) return null;
  return request.mode === "add" ? (
    <SidePanel open onClose={onClose} label={texts.editor.subtitles.add[request.kind]}>
      <EditorForm
        initial={blankValues(request.kind, request.parent)}
        details={undefined}
        focusOnMount={false}
        onClose={onClose}
        onSaved={onSaved}
      />
    </SidePanel>
  ) : (
    // A new key for each node: nothing of one editor survives into the next.
    <ExistingNodeEditor key={request.id} id={request.id} onClose={onClose} onSaved={onSaved} />
  );
}

function ExistingNodeEditor({
  id,
  onClose,
  onSaved,
}: { id: string } & Omit<NodeEditorProps, "request">) {
  const t = texts.editor;
  const { data: details, error } = useQuery(nodeDetailsQuery(id));
  // The panel gives the focus when it opens; a form that comes after the
  // loading takes it itself.
  const [loadedInPanel] = useState(details === undefined);
  const kind = details ? editorKindOf(details) : null;
  const label = details ? interpolate(t.label.edit, { name: details.name }) : t.label.loading;
  const problem = error
    ? presentFailure(error).message
    : details && kind === null
      ? t.unsupported
      : undefined;

  return (
    <SidePanel open onClose={onClose} label={label}>
      {details && kind ? (
        // The form starts from the first detail; the later ones are merged in.
        <EditorForm
          initial={valuesOf(kind, details)}
          details={details}
          focusOnMount={loadedInPanel}
          onClose={onClose}
          onSaved={onSaved}
        />
      ) : (
        <div className={styles.loading}>
          <p role={problem ? "alert" : "status"}>{problem ?? t.loading}</p>
          <Button onClick={onClose}>{t.cancel}</Button>
        </div>
      )}
    </SidePanel>
  );
}
