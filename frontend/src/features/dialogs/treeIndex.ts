import type { TreeView_Serialize } from "../../bindings";
import {
  ancestorsOf,
  indexTree,
  readableNames,
  type TreeEntry,
  type TreeIndex,
} from "../tree/treeModel";

/** A node of the tree with where it sits, for the pickers and the editor. */
export interface IndexedNode extends TreeEntry {
  /** "Clienti / ACME / web01". */
  path: string;
}

/**
 * Where `entry` sits, as the interface writes a path. The names of the tree
 * are plain text, not a path of the core to parse.
 */
function locationOf(index: TreeIndex, entry: TreeEntry): string {
  const folders = ancestorsOf(index, entry.node.id)
    .reverse()
    .map((id) => index.entries.get(id)?.node.name ?? "");
  return readableNames([...folders, entry.node.name]);
}

/** Every node of `tree` in tree order (folders before their content), with its path. */
export function indexedNodes(tree: TreeView_Serialize): IndexedNode[] {
  const index = indexTree(tree);
  return [...index.entries.values()].map((entry) => ({
    ...entry,
    path: locationOf(index, entry),
  }));
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
