import type { TreeNode_Serialize, TreeView_Serialize } from "../../bindings";

function find(
  nodes: readonly TreeNode_Serialize[],
  matches: (node: TreeNode_Serialize) => boolean,
): TreeNode_Serialize | null {
  for (const node of nodes) {
    if (matches(node)) return node;
    const inside = find(node.children, matches);
    if (inside) return inside;
  }
  return null;
}

/**
 * The node that a line of the import report talks about, by its id in the
 * source (SPEC-UI §5.4). The same file imported twice with "Nuovi ID" gives
 * two nodes with the same source id: the one under the folder of this import
 * wins. With "Aggiorna" the updated nodes stay where they were, maybe
 * outside that folder: then the whole tree is searched.
 */
export function importedNode(
  tree: TreeView_Serialize,
  importRoot: string,
  sourceId: string,
): string | null {
  const bySource = (node: TreeNode_Serialize) => node.sourceId === sourceId;
  const root = find(tree.nodes, (node) => node.id === importRoot);
  const found = (root ? find([root], bySource) : null) ?? find(tree.nodes, bySource);
  return found?.id ?? null;
}
