import { useWindowActions } from "../../app/windowActions";
import { NodePicker, type PickerNode } from "../../components/NodePicker";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import type { TreeIndex } from "./treeModel";

interface MoveToDialogProps {
  index: TreeIndex;
  /** The node to move; none with the dialog closed. */
  id: string | undefined;
  onClose: () => void;
}

/**
 * "Sposta in…": the keyboard alternative to dragging a node (SPEC-UI §5.5). The
 * picker offers the folders and the root; the node itself and what it
 * contains cannot be chosen. The node goes last in the folder chosen.
 */
export function MoveToDialog({ index, id, onClose }: MoveToDialogProps) {
  const { move } = useWindowActions();
  const t = texts.tree.move;
  const entry = id === undefined ? undefined : index.entries.get(id);
  const nodes: PickerNode[] = [...index.entries.values()].map(({ node, kind, depth }) => ({
    id: node.id,
    name: node.name,
    kind,
    depth,
  }));
  return (
    <NodePicker
      open={entry !== undefined}
      onClose={onClose}
      title={interpolate(t.title, { name: entry?.node.name ?? "" })}
      filterLabel={t.filter}
      note={entry?.kind === "folder" ? t.note : undefined}
      nodes={nodes}
      choose="folders"
      exclude={id}
      onChoose={(destination) => {
        if (id !== undefined) move(id, destination, { kind: "end" });
        onClose();
      }}
    />
  );
}
