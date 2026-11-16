import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { type AppStatus_Serialize, commands, type DocumentSource } from "../bindings";
import { appStatusQuery } from "../ipc/queries";
import { IpcFailure, type IpcError, unwrap } from "../ipc/result";
import { reportFailure } from "./failures";

/** What the window shows. */
export type StartupView =
  | { view: "loading" }
  | { view: "failed"; error: unknown }
  | { view: "firstRun"; status: AppStatus_Serialize }
  | { view: "document"; status: AppStatus_Serialize };

/**
 * The document to open at startup: the one opened last, else the default one
 * of the configuration, if their file exists; none means the first run
 * (inventario §5.12: the first run appears when there is no document).
 */
export function startupSource(status: AppStatus_Serialize): DocumentSource | null {
  if (status.lastDocument?.exists) {
    return { kind: "recent", path: status.lastDocument.path };
  }
  return status.defaultDocument.exists ? { kind: "default" } : null;
}

/**
 * The opening of the document at startup, once: the backend starts with none
 * open. A failure is kept as the result, to show it, and the window goes on
 * to the first run.
 */
const startupQuery = queryOptions({
  queryKey: ["startup"],
  queryFn: async (): Promise<{ failure: IpcError | null }> => {
    const status = unwrap(await commands.appStatus());
    const source = status.document === null ? startupSource(status) : null;
    if (source === null) return { failure: null };
    const opened = await commands.openDocument(source);
    return { failure: opened.status === "error" ? opened.error : null };
  },
  staleTime: Infinity,
  gcTime: Infinity,
});

/** The status of the app, after the opening of the document at startup. */
export function useStartup(): StartupView {
  const queryClient = useQueryClient();
  const startup = useQuery(startupQuery);
  const status = useQuery({ ...appStatusQuery, enabled: startup.isSuccess });
  const failure = startup.data?.failure ?? null;

  useEffect(() => {
    if (failure !== null) reportFailure(new IpcFailure(failure), queryClient);
  }, [failure, queryClient]);

  if (startup.isError) return { view: "failed", error: startup.error };
  if (status.isError) return { view: "failed", error: status.error };
  if (!status.isSuccess) return { view: "loading" };
  return status.data.document === null
    ? { view: "firstRun", status: status.data }
    : { view: "document", status: status.data };
}
