import type { DeletePreview } from "../../bindings";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { pathName, type TreeIndex } from "../tree/treeModel";

/** A row of "È usata da N nodi": who refers to what is deleted, and how. */
export interface UsageRow {
  key: string;
  /** The node that refers to it, to go to it. */
  id: string;
  path: string;
  relation: string;
}

/** What deleting changes for the nodes outside the subtree (inventario §5.11). */
export interface DeleteUsage {
  /** The direct references (DEC-018.1). */
  rows: UsageRow[];
  /** The connections that receive one of its credentials by inheritance (ambiguity 13). */
  inherited: number;
  /** Some user loses a credential: it becomes "Chiedi all'avvio" (DEC-016.5). */
  credentialsUsed: boolean;
  /** Some other reference (a jump host) is left dangling for the validation. */
  otherReferences: boolean;
}

/**
 * Who uses what `preview` deletes, with the kinds of the nodes from `index`:
 * a credential referred to by a folder is its "credenziale predefinita", by
 * another node "collegata"; any other reference "fa riferimento a …".
 */
export function deleteUsage(preview: DeletePreview, index: TreeIndex | undefined): DeleteUsage {
  const t = texts.details.remove;
  const kindOf = (id: string) => index?.entries.get(id)?.kind;
  let credentialsUsed = preview.inherited.length > 0;
  let otherReferences = false;
  const rows = preview.references.map((reference) => {
    const credential =
      kindOf(reference.target) === "credential" ||
      (reference.target === preview.id && preview.node_type.type === "credential");
    credentialsUsed ||= credential;
    otherReferences ||= !credential;
    // The name of a node is untrusted text (SPEC-UI §1.3).
    const target = visibleText(pathName(reference.target_path));
    const fromFolder = kindOf(reference.from) === "folder";
    const relation = !credential
      ? interpolate(t.references, { target })
      : reference.target !== preview.id
        ? // A credential inside the deleted folder: which one.
          interpolate(fromFolder ? t.folderDefaultOf : t.linkedTo, { target })
        : fromFolder
          ? t.folderDefault
          : t.linked;
    return {
      key: `${reference.from}-${reference.target}`,
      id: reference.from,
      path: reference.from_path,
      relation,
    };
  });
  return { rows, inherited: preview.inherited.length, credentialsUsed, otherReferences };
}
