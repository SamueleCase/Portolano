import { useQuery } from "@tanstack/react-query";

import { NodePicker, type PickerNode } from "../../components/NodePicker";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { type IndexedNode, indexedNodes } from "./treeIndex";

/** What the picker offers: SSH jump hosts or credentials (inventario §5.8). */
export type PickerPurpose = "jumpHost" | "credential";

interface TreeNodePickerProps {
  open: boolean;
  purpose: PickerPurpose;
  /** The node being edited: shown, never choosable (a jump host cannot be itself). */
  exclude?: string | undefined;
  onChoose: (id: string) => void;
  onClose: () => void;
}

function offered(entry: IndexedNode, purpose: PickerPurpose): boolean {
  return purpose === "jumpHost" ? entry.kind === "ssh" : entry.kind === "credential";
}

/** The `NodePicker` over the open document, filtered for `purpose`. */
export function TreeNodePicker({ open, purpose, exclude, onChoose, onClose }: TreeNodePickerProps) {
  const t = texts.dialogs.picker;
  const { data: tree } = useQuery({ ...treeQuery, enabled: open });
  const entries = tree ? indexedNodes(tree) : [];
  const nodes: PickerNode[] = entries
    .filter((entry) => entry.kind === "folder" || offered(entry, purpose))
    .map((entry) => ({
      id: entry.node.id,
      name: entry.node.name,
      kind: entry.kind,
      depth: entry.depth,
      // The mono line: the address of a connection, the path of a credential.
      host:
        entry.kind === "folder"
          ? undefined
          : entry.kind === "credential"
            ? entry.path
            : entry.node.summary || undefined,
      excluded: entry.node.id === exclude,
    }));
  const excluded = entries.find((entry) => entry.node.id === exclude && offered(entry, purpose));
  return (
    <NodePicker
      open={open}
      onClose={onClose}
      onChoose={onChoose}
      title={t[purpose].title}
      filterLabel={t[purpose].filter}
      nodes={nodes}
      note={
        excluded
          ? interpolate(t.selfExcluded, { name: visibleText(excluded.node.name) })
          : undefined
      }
    />
  );
}
