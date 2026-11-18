import type { TreeNode_Serialize, TreeView_Serialize } from "../../bindings";
import { readableNames } from "../../components/readablePath";

/**
 * The readable path of every node of `tree`, "Produzione / web01" (glossario
 * §6: " / " between the names, no leading "/"), by id. The names stay as the
 * document has them: the components that show a path make it visible.
 */
export function readablePaths(tree: TreeView_Serialize): Map<string, string> {
  const paths = new Map<string, string>();
  const visit = (nodes: readonly TreeNode_Serialize[], folders: readonly string[]) => {
    for (const node of nodes) {
      const names = [...folders, node.name];
      paths.set(node.id, readableNames(names));
      visit(node.children, names);
    }
  };
  visit(tree.nodes, []);
  return paths;
}
