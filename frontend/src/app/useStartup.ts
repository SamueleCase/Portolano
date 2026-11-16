import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { type AppStatus_Serialize, commands, type DocumentSource } from "../bindings";
import { appStatusQuery } from "../ipc/queries";
import { IpcFailure, type IpcError, unwrap } from "../ipc/result";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { showToast } from "../state/toasts";
import { reportFailure } from "./failures";

/** What the window shows. */
export type StartupView =
  | { view: "loading" }
  | { view: "failed"; error: unknown }
  | { view: "firstRun"; status: AppStatus_Serialize }
  | { view: "document"; status: AppStatus_Serialize };

/**
 * The documents to try at startup, in order: the one opened last, then the
 * default one of the configuration (DEC-041.7), those whose file exists;
 * none means the first run (inventario §5.12: the first run appears when
 * there is no document).
 */
export function startupSources(status: AppStatus_Serialize): DocumentSource[] {
  const sources: DocumentSource[] = [];
  if (status.lastDocument?.exists) sources.push({ kind: "recent", path: status.lastDocument.path });
  const lastIsDefault = status.lastDocument?.path === status.defaultDocument.path;
  if (status.defaultDocument.exists && !(lastIsDefault && sources.length > 0)) {
    sources.push({ kind: "default" });
  }
  return sources;
}

/**
 * The opening of the document at startup, once: the backend starts with none
 * open. The first source that opens wins. The failure of an earlier one is
 * kept, to say why another document opened (DEC-041.7); if none opens, the
 * last failure is shown and the window goes on to the first run.
 */
const startupQuery = queryOptions({
  queryKey: ["startup"],
  queryFn: async (): Promise<{ failure: IpcError | null; opened: boolean }> => {
    const status = unwrap(await commands.appStatus());
    if (status.document !== null) return { failure: null, opened: true };
    let failure: IpcError | null = null;
    for (const source of startupSources(status)) {
      const result = await commands.openDocument(source);
      if (result.status === "ok") return { failure, opened: true };
      failure = result.error;
    }
    return { failure, opened: false };
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
  const opened = startup.data?.opened ?? false;

  useEffect(() => {
    if (failure === null) return;
    if (opened) {
      // The last document did not open: the default one did instead.
      showToast({
        kind: "info",
        text: interpolate(texts.app.toasts.fellBack, { message: failure.message }),
      });
    } else {
      reportFailure(new IpcFailure(failure), queryClient);
    }
  }, [failure, opened, queryClient]);

  if (startup.isError) return { view: "failed", error: startup.error };
  // A refetch that fails later leaves the window as it is: only missing data fails it.
  if (status.data === undefined) {
    return status.isError ? { view: "failed", error: status.error } : { view: "loading" };
  }
  return status.data.document === null
    ? { view: "firstRun", status: status.data }
    : { view: "document", status: status.data };
}
