import type {
  IconView,
  PlacementInput,
  TreeNode_Serialize,
  TreeView_Serialize,
} from "../../bindings";
import type { NodeIconChoice } from "../../components/icons/NodeIcon";
import type { NodeKind } from "../../components/nodeKind";
import type { TreeFilter } from "../../state/workspace";

/** A node of the tree with what the window needs to find it. */
export interface TreeEntry {
  node: TreeNode_Serialize;
  kind: NodeKind;
  /** The folder that holds it; `null` at the root. */
  parentId: string | null;
  /** 0 at the root. */
  depth: number;
}

/** The tree of the document, by id. */
export interface TreeIndex {
  entries: ReadonlyMap<string, TreeEntry>;
  rootIds: readonly string[];
}

/** The kind the interface draws for `node`. */
export function nodeKindOf(node: Pick<TreeNode_Serialize, "kind" | "protocol">): NodeKind {
  if (node.kind === "connection") {
    return node.protocol === "rdp" ? "rdp" : "ssh";
  }
  return node.kind;
}

/** The icon of a node as the components take it; none is the icon of its kind. */
export function iconChoice(icon: IconView | null): NodeIconChoice | undefined {
  if (icon === null) return undefined;
  if (icon.kind === "builtin") return { source: "builtin", name: icon.name };
  // The scheme serves `<sha256>.<ext>`: the last segment of the URL.
  return { source: "custom", fileName: icon.url.split("/").pop() ?? "" };
}

/** Indexes `view` by id, in tree order. */
export function indexTree(view: TreeView_Serialize): TreeIndex {
  const entries = new Map<string, TreeEntry>();
  const visit = (nodes: readonly TreeNode_Serialize[], parentId: string | null, depth: number) => {
    for (const node of nodes) {
      entries.set(node.id, { node, kind: nodeKindOf(node), parentId, depth });
      visit(node.children, node.id, depth + 1);
    }
  };
  visit(view.nodes, null, 0);
  return { entries, rootIds: view.nodes.map((node) => node.id) };
}

/** How many nodes the document has, folders included (the footer of the vault). */
export function nodeCount(index: TreeIndex): number {
  return index.entries.size;
}

function passesFilter(entry: TreeEntry, filter: TreeFilter): boolean {
  switch (filter) {
    case "all":
      return true;
    case "ssh":
    case "rdp":
      return entry.kind === filter;
    case "credentials":
      return entry.kind === "credential";
    case "favorites":
      return entry.node.favorite;
  }
}

/**
 * The nodes the tree shows for `filter` and the ids the search found
 * (`null`: no search): the nodes that are not folders and pass both, and the
 * folders that contain one (inventario §5.1). `null` when nothing filters.
 */
export function visibleIds(
  index: TreeIndex,
  filter: TreeFilter,
  found: ReadonlySet<string> | null,
): ReadonlySet<string> | null {
  if (filter === "all" && found === null) return null;
  const visible = new Set<string>();
  for (const [id, entry] of index.entries) {
    if (entry.kind === "folder" || !passesFilter(entry, filter)) continue;
    if (found !== null && !found.has(id)) continue;
    for (let current: string | null = id; current !== null;) {
      if (visible.has(current)) break;
      visible.add(current);
      current = index.entries.get(current)?.parentId ?? null;
    }
  }
  return visible;
}

/**
 * Where a new node goes (ambiguity 10): into the selected folder, else into
 * the folder of the selected node, else at the root.
 */
export function parentForNew(index: TreeIndex, selectedId: string | null): string | null {
  const selected = selectedId === null ? undefined : index.entries.get(selectedId);
  if (selected === undefined) return null;
  return selected.kind === "folder" ? selected.node.id : selected.parentId;
}

/** The folders that hold `id`, from its parent up to the root. */
export function ancestorsOf(index: TreeIndex, id: string): string[] {
  const ancestors: string[] = [];
  for (let current = index.entries.get(id)?.parentId ?? null; current !== null;) {
    ancestors.push(current);
    current = index.entries.get(current)?.parentId ?? null;
  }
  return ancestors;
}

/** Whether `id` is `ancestor` or inside it. */
export function isWithin(index: TreeIndex, id: string, ancestor: string): boolean {
  for (let current: string | null = id; current !== null;) {
    if (current === ancestor) return true;
    current = index.entries.get(current)?.parentId ?? null;
  }
  return false;
}

/** The names of a path of the core (`/Clienti/ACME\/Nord`, with `/` and `\` escaped). */
export function pathSegments(path: string): string[] {
  const segments: string[] = [];
  let current = "";
  let escaped = false;
  for (const character of path) {
    if (escaped) {
      current += character;
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (character === "/") {
      if (current !== "" || segments.length > 0) segments.push(current);
      current = "";
    } else {
      current += character;
    }
  }
  segments.push(current);
  return segments;
}

/** Names from the root, as the interface writes a path: "Clienti / ACME/Nord / web01" (inventario §6). */
export function readableNames(names: readonly string[]): string {
  return names.join(" / ");
}

/** A path of the core as the interface writes it (`readableNames`). */
export function readablePath(path: string): string {
  return readableNames(pathSegments(path));
}

/** The name at the end of a path of the core. */
export function pathName(path: string): string {
  return pathSegments(path).at(-1) ?? "";
}

/**
 * The host of `[user@]host[:port]` (`[…]` around an IPv6 host): the summary
 * of a connection, the meta of its row (inventario §4.19).
 */
export function summaryHost(summary: string): string {
  const host = summary.slice(summary.lastIndexOf("@") + 1);
  if (host.startsWith("[")) {
    const end = host.indexOf("]");
    return end === -1 ? host.slice(1) : host.slice(1, end);
  }
  const colon = host.indexOf(":");
  return colon === -1 ? host : host.slice(0, colon);
}

/** The children of the folder that holds `id` (the root's for a root node), by id. */
export function siblingsOf(index: TreeIndex, id: string): readonly string[] {
  const parentId = index.entries.get(id)?.parentId ?? null;
  if (parentId === null) return index.rootIds;
  return index.entries.get(parentId)?.node.children.map((child) => child.id) ?? [];
}

/**
 * Where node `id` goes one place up (`-1`) or down (`1`) among its siblings
 * shown (`visible`, `null` for all: a search or a filter hides some), as
 * `move_node` takes it; `undefined` at the edge.
 */
export function siblingPlacement(
  index: TreeIndex,
  id: string,
  step: -1 | 1,
  visible: ReadonlySet<string> | null = null,
): PlacementInput | undefined {
  const siblings = siblingsOf(index, id).filter(
    (sibling) => visible === null || visible.has(sibling),
  );
  const neighbor = siblings[siblings.indexOf(id) + step];
  if (neighbor === undefined) return undefined;
  return step < 0 ? { kind: "before", id: neighbor } : { kind: "after", id: neighbor };
}
