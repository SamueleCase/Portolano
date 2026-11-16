import type { QueryClient } from "@tanstack/react-query";

import { readablePath } from "../features/tree/treeModel";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { presentFailure, type PresentedError } from "../ipc/errors";
import { queryKeys } from "../ipc/keys";
import { appStatusQuery } from "../ipc/queries";
import { requestUnlock } from "../state/requests";
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
 * the banners of the document (alone, without a toast), the first run when
 * there is no document, and a persistent error toast for the rest (ambiguity
 * 17). The errors under a field and the confirmation of a deletion are shown
 * by the screens that have them; here they become toasts.
 */
export function reportFailure(failure: unknown, queryClient: QueryClient): void {
  const error = presentFailure(failure);
  const fileName = queryClient.getQueryData(appStatusQuery.queryKey)?.document?.fileName;
  const { setBanner } = useWorkspace.getState();
  switch (error.presentation) {
    case "reloadBanner":
      if (fileName) return setBanner({ kind: "modified", fileName });
      break;
    case "missingBanner":
      if (fileName) return setBanner({ kind: "missing", fileName });
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

/** Whether `failure` is a command refused because the vault is locked. */
export function isVaultLocked(failure: unknown): boolean {
  return presentFailure(failure).presentation === "unlockVault";
}

/**
 * Runs `command`; if it fails because the vault is locked and the unlock
 * dialog is mounted (`canUnlock`), asks to unlock it and, once unlocked, runs
 * the command again (DEC-041.2). Any other failure, or a second one, goes to
 * `reportFailure`.
 */
export async function runReported(
  command: () => Promise<unknown>,
  queryClient: QueryClient,
  canUnlock: () => boolean,
): Promise<void> {
  try {
    await command();
  } catch (failure) {
    if (isVaultLocked(failure) && canUnlock() && (await requestUnlock())) {
      return runReported(command, queryClient, () => false);
    }
    reportFailure(failure, queryClient);
  }
}
