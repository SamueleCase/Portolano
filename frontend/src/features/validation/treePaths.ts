import type { TreeNode_Serialize, TreeView_Serialize } from "../../bindings";
import { texts } from "../../i18n/it";

/**
 * The readable path of every node of `tree`, "Produzione / web01" (glossario
 * §6: " / " between the names, no leading "/"), by id. The names stay as the
 * document has them: the components that show a path make it visible.
 */
export function readablePaths(tree: TreeView_Serialize): Map<string, string> {
  const paths = new Map<string, string>();
  const visit = (nodes: readonly TreeNode_Serialize[], parent: string | null) => {
    for (const node of nodes) {
      const path =
        parent === null ? node.name : `${parent}${texts.validation.separator}${node.name}`;
      paths.set(node.id, path);
      visit(node.children, path);
    }
  };
  visit(tree.nodes, null);
  return paths;
}
