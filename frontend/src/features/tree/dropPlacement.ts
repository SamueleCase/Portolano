import type { PlacementInput } from "../../bindings";

/**
 * Where a node dropped at `childIndex` among the shown children `siblings` of
 * its new folder goes, as `move_node` takes it. The tree may show only some
 * children (a search, a filter): the place is given by the shown neighbors,
 * not by an index of the full list. The dragged node is not its own
 * neighbor.
 */
export function dropPlacement(
  siblings: readonly string[],
  childIndex: number,
  dragged: string,
): PlacementInput {
  const after = siblings.slice(childIndex).find((id) => id !== dragged);
  if (after !== undefined) return { kind: "before", id: after };
  const before = siblings
    .slice(0, childIndex)
    .reverse()
    .find((id) => id !== dragged);
  return before === undefined ? { kind: "end" } : { kind: "after", id: before };
}
