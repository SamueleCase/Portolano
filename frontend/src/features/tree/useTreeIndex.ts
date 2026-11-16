import { useQuery } from "@tanstack/react-query";

import { treeQuery } from "../../ipc/queries";
import { useWorkspace } from "../../state/workspace";
import { indexTree, type TreeEntry } from "./treeModel";

/** The tree of the open document, indexed by id (recomputed only when it changes). */
export function useTreeIndex() {
  return useQuery({ ...treeQuery, select: indexTree });
}

/** The selected node, if it is in the tree. */
export function useSelectedEntry(): TreeEntry | undefined {
  const selectedId = useWorkspace((state) => state.selectedId);
  const index = useTreeIndex().data;
  return selectedId === null ? undefined : index?.entries.get(selectedId);
}
