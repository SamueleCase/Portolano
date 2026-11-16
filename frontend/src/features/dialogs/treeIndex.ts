import type { NodeKind } from "../../components/nodeKind";
import type { TreeNode_Serialize, TreeView_Serialize } from "../../bindings";

/** A node of the tree with where it sits, for the pickers and the editor. */
export interface IndexedNode {
  node: TreeNode_Serialize;
  kind: NodeKind;
  depth: number;
  /** The folder that contains it; `null` for the root. */
  parent: string | null;
  /** "Clienti / ACME / web01". */
  path: string;
}

/** How the interface draws a node of `kind` and `protocol`. */
export function nodeKindOf(node: Pick<TreeNode_Serialize, "kind" | "protocol">): NodeKind {
  if (node.kind !== "connection") return node.kind;
  return node.protocol === "rdp" ? "rdp" : "ssh";
}

/** Every node of `tree` in tree order (folders before their content). */
export function indexTree(tree: TreeView_Serialize): IndexedNode[] {
  const entries: IndexedNode[] = [];
  const visit = (nodes: TreeNode_Serialize[], parent: IndexedNode | null) => {
    for (const node of nodes) {
      const entry: IndexedNode = {
        node,
        kind: nodeKindOf(node),
        depth: parent ? parent.depth + 1 : 0,
        parent: parent?.node.id ?? null,
        path: parent ? `${parent.path} / ${node.name}` : node.name,
      };
      entries.push(entry);
      visit(node.children, entry);
    }
  };
  visit(tree.nodes, null);
  return entries;
}

/**
 * The credentials a name refers to, as SPEC §4.3 resolves a credential "Per
 * nome": in the whole document, those with exactly that name, or else those
 * whose name differs only by case. It is only the preview of the selector
 * ("Oggi corrisponde a"): the backend decides at each connection and in the
 * validation.
 */
export function credentialsNamed(entries: readonly IndexedNode[], name: string): IndexedNode[] {
  const credentials = entries.filter((entry) => entry.kind === "credential");
  const exact = credentials.filter((entry) => entry.node.name === name);
  if (exact.length > 0) return exact;
  const folded = name.toLowerCase();
  return credentials.filter((entry) => entry.node.name.toLowerCase() === folded);
}
