import type { NodeKind } from "./nodeKind";

/** A node of the picker, in tree order (folders before their content). */
export interface PickerNode {
  id: string;
  name: string;
  kind: NodeKind;
  depth: number;
  host?: string | undefined;
  /** Shown but not choosable, such as the connection being edited. */
  excluded?: boolean;
}

/** A row of the list: a node or, choosing folders, the root of the document. */
export interface PickerRow {
  /** Unique among the rows, for the id of the option. */
  key: string;
  /** What `onChoose` receives: `null` is the root of the document. */
  value: string | null;
  name: string;
  kind: NodeKind;
  /** The level of indentation: choosing folders the root is 0. */
  depth: number;
  host?: string | undefined;
  /** A folder that only heads the nodes it contains, not an option. */
  heading: boolean;
  disabled: boolean;
  /** Passes the search; false for a folder shown only because it leads to a result. */
  match: boolean;
  /** The root and the folders that contain the row, from the top; empty for the root. */
  path: readonly string[];
}

function contains(texts: readonly string[], search: string): boolean {
  const needle = search.trim().toLocaleLowerCase("it");
  return texts.some((text) => text.toLocaleLowerCase("it").includes(needle));
}

interface VisibleNode {
  node: PickerNode;
  match: boolean;
  /** The names of the folders that contain the node, from the top. */
  ancestors: readonly string[];
}

/** For each node, the names of the folders that contain it. */
function ancestorNames(nodes: readonly PickerNode[]): string[][] {
  const open: string[] = [];
  return nodes.map((node) => {
    open.length = Math.min(open.length, node.depth);
    const ancestors = [...open];
    open.push(node.name);
    return ancestors;
  });
}

/** The nodes that pass `matches`, with the folders that lead to them. */
function visibleNodes(
  nodes: readonly PickerNode[],
  matches: (node: PickerNode) => boolean,
): VisibleNode[] {
  const ancestors = ancestorNames(nodes);
  const matched = new Set<number>();
  const shown = new Set<number>();
  nodes.forEach((node, index) => {
    if (!matches(node)) return;
    matched.add(index);
    shown.add(index);
    let depth = node.depth;
    for (let parent = index - 1; parent >= 0 && depth > 0; parent--) {
      const candidate = nodes[parent];
      if (candidate && candidate.depth < depth) {
        shown.add(parent);
        depth = candidate.depth;
      }
    }
  });
  return nodes.flatMap((node, index) =>
    shown.has(index)
      ? [{ node, match: matched.has(index), ancestors: ancestors[index] ?? [] }]
      : [],
  );
}

/** The ids of the node `id` and of its descendants, which follow it at a greater depth. */
function subtree(nodes: readonly PickerNode[], id: string | undefined): Set<string> {
  const ids = new Set<string>();
  const start = nodes.findIndex((node) => node.id === id);
  const top = nodes[start];
  if (!top) return ids;
  ids.add(top.id);
  for (const node of nodes.slice(start + 1)) {
    if (node.depth <= top.depth) break;
    ids.add(node.id);
  }
  return ids;
}

function nodeRow(
  { node, match, ancestors }: VisibleNode,
  root: string,
  depth: number,
  disabled: boolean,
  heading = false,
): PickerRow {
  const { id, name, kind, host } = node;
  // The prefix keeps the key of a node apart from the key of the root.
  const path = [root, ...ancestors];
  return { key: `node-${id}`, value: id, name, kind, depth, host, heading, disabled, match, path };
}

/** The rows that choose a connection or a credential, headed by their folders. */
export function nodeRows(nodes: readonly PickerNode[], search: string, root: string): PickerRow[] {
  return visibleNodes(
    nodes,
    (node) => node.kind !== "folder" && contains([node.name, node.host ?? ""], search),
  ).map((visible) =>
    nodeRow(
      visible,
      root,
      visible.node.depth,
      Boolean(visible.node.excluded),
      visible.node.kind === "folder",
    ),
  );
}

/**
 * The rows that choose a folder, empty ones included, after the root of the
 * document (depth 0, so the folders move one level in). The folders of the
 * subtree `exclude` are shown disabled.
 */
export function folderRows(
  nodes: readonly PickerNode[],
  search: string,
  root: string,
  exclude: string | undefined,
): PickerRow[] {
  const excluded = subtree(nodes, exclude);
  const folders = visibleNodes(
    nodes,
    (node) => node.kind === "folder" && contains([node.name], search),
  ).map((visible) =>
    nodeRow(
      visible,
      root,
      visible.node.depth + 1,
      Boolean(visible.node.excluded) || excluded.has(visible.node.id),
    ),
  );
  const rootRow: PickerRow = {
    key: "root",
    value: null,
    name: root,
    kind: "folder",
    depth: 0,
    heading: false,
    disabled: false,
    match: true,
    path: [],
  };
  return contains([root], search) ? [rootRow, ...folders] : folders;
}
