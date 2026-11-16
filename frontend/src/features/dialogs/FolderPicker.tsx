import { useQuery } from "@tanstack/react-query";

import { NodePicker } from "../../components/NodePicker";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { indexTree } from "./treeIndex";

interface FolderPickerProps {
  open: boolean;
  /**
   * The node being edited: shown but not choosable with its descendants,
   * since a folder cannot go inside itself.
   */
  exclude?: string | undefined;
  /** The folder chosen; `null` is the root of the document. */
  onChoose: (folder: string | null) => void;
  onClose: () => void;
}

/** The folders of the open document and its root, to choose where a node goes. */
export function FolderPicker({ open, exclude, onChoose, onClose }: FolderPickerProps) {
  const t = texts.dialogs.picker.folder;
  const { data: tree } = useQuery({ ...treeQuery, enabled: open });
  const nodes = (tree ? indexTree(tree) : [])
    .filter((entry) => entry.kind === "folder")
    .map((entry) => ({
      id: entry.node.id,
      name: entry.node.name,
      kind: entry.kind,
      depth: entry.depth,
    }));
  return (
    <NodePicker
      open={open}
      onClose={onClose}
      title={t.title}
      filterLabel={t.filter}
      nodes={nodes}
      choose="folders"
      exclude={exclude}
      onChoose={onChoose}
    />
  );
}
