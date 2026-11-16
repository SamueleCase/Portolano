import type { QueryClient } from "@tanstack/react-query";

import { readablePath } from "../features/tree/treeModel";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { presentFailure, type PresentedError } from "../ipc/errors";
import { queryKeys } from "../ipc/keys";
import { appStatusQuery } from "../ipc/queries";
import { showToast } from "../state/toasts";
import { useWorkspace } from "../state/workspace";

/** The text of the error toast of `error`, with the candidates of an ambiguous name. */
export function errorText(error: PresentedError): string {
  const message =
    error.details?.kind === "ambiguous"
      ? interpolate(texts.app.toasts.candidates, {
          message: error.message,
          candidates: error.details.candidates.map((c) => readablePath(c.path)).join(", "),
        })
      : error.message;
  return interpolate(texts.app.toasts.error, { title: error.title, message });
}

/**
 * Shows a failed command where its code says (SPEC-UI §4.2, `ipc/errors.ts`):
 * the banners of the document, the first run when there is no document, and
 * a persistent error toast for the rest (ambiguity 17). The errors under a
 * field and the confirmation of a deletion are shown by the screens that
 * have them; here they become toasts too.
 */
export function reportFailure(failure: unknown, queryClient: QueryClient): void {
  const error = presentFailure(failure);
  const { setBanner } = useWorkspace.getState();
  switch (error.presentation) {
    case "reloadBanner":
      setBanner({ kind: "modified", fileName: currentFileName(queryClient) });
      break;
    case "missingBanner":
      setBanner({ kind: "missing", fileName: currentFileName(queryClient) });
      break;
    case "firstRun":
      // The document is gone: the window goes back to the first run.
      void queryClient.invalidateQueries({ queryKey: queryKeys.appStatus });
      return;
    default:
      break;
  }
  showToast({ kind: "error", text: errorText(error) });
}

/** The file name of the open document, as the status of the app has it. */
function currentFileName(queryClient: QueryClient): string {
  return queryClient.getQueryData(appStatusQuery.queryKey)?.document?.fileName ?? "";
}
