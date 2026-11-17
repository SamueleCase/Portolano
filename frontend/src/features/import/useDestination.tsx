import { useQuery } from "@tanstack/react-query";
import { type ReactNode, useMemo, useState } from "react";

import type { TreeNode_Serialize } from "../../bindings";
import { NodePicker, type PickerNode } from "../../components/NodePicker";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { readablePaths } from "../validation/treePaths";

/** The folders of the tree in tree order, for the picker (DEC-038.1). */
function folders(nodes: readonly TreeNode_Serialize[], depth = 0): PickerNode[] {
  return nodes
    .filter((node) => node.kind === "folder")
    .flatMap((node) => [
      { id: node.id, name: node.name, kind: "folder" as const, depth },
      ...folders(node.children, depth + 1),
    ]);
}

/**
 * The destination folder of step 2: its readable path ("Radice del
 * documento" for the root) and the picker of the folders that changes it.
 */
export function useDestination(
  into: string | null,
  onChange: (into: string | null) => void,
): { label: string; choose: () => void; picker: ReactNode } {
  const tree = useQuery(treeQuery);
  const [picking, setPicking] = useState(false);
  const { options } = texts.importWizard;
  const paths = useMemo(
    () => (tree.data ? readablePaths(tree.data) : new Map<string, string>()),
    [tree.data],
  );
  const nodes = useMemo(() => folders(tree.data?.nodes ?? []), [tree.data]);
  const label = into === null ? options.root : (paths.get(into) ?? options.folderMissing);
  const picker = (
    <NodePicker
      open={picking}
      onClose={() => setPicking(false)}
      title={options.pickerTitle}
      filterLabel={options.pickerFilter}
      nodes={nodes}
      choose="folders"
      onChoose={(id) => {
        setPicking(false);
        onChange(id);
      }}
    />
  );
  return { label, choose: () => setPicking(true), picker };
}
