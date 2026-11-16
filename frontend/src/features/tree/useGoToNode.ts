import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { treeQuery } from "../../ipc/queries";
import { useWorkspace } from "../../state/workspace";
import { ancestorsOf, indexTree } from "./treeModel";

/**
 * "Vai al nodo": selects node `id`, opens the folders that hold it (as the
 * tree has them now) and shows its detail.
 */
export function revealNode(queryClient: QueryClient, id: string): void {
  const view = queryClient.getQueryData(treeQuery.queryKey);
  useWorkspace.getState().reveal(id, view ? ancestorsOf(indexTree(view), id) : []);
}

/** `revealNode` for the usages of the detail, the validation and the import report. */
export function useGoToNode(): (id: string) => void {
  const queryClient = useQueryClient();
  return useCallback((id: string) => revealNode(queryClient, id), [queryClient]);
}
