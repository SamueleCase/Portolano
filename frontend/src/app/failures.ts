import type { QueryClient } from "@tanstack/react-query";

import { readablePath } from "../features/tree/treeModel";
import { interpolate } from "../i18n/format";
import { texts } from "../i18n/it";
import { presentFailure, type PresentedError } from "../ipc/errors";
import { queryKeys } from "../ipc/keys";
import { appStatusQuery, vaultStatusQuery } from "../ipc/queries";
import { requestVault } from "../state/requests";
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
 * Whether the dialog that readies the vault is mounted: the creation when its
 * file does not exist yet, else the unlock.
 */
export type CanReadyVault = (dialog: "unlockVault" | "createFileVault") => boolean;

/**
 * Makes sure the file vault is open before a command that needs it: with the
 * vault locked and the dialog it needs mounted, asks for it (`requestVault`).
 * Resolves to whether to go on: `false` only when the user gave up. Without
 * the dialog the command goes on, and says itself that the vault is locked.
 */
export async function vaultReady(
  queryClient: QueryClient,
  canAsk: CanReadyVault,
): Promise<boolean> {
  const vault = await queryClient.fetchQuery({ ...vaultStatusQuery, staleTime: 0 });
  if (!vault.locked || !canAsk(vault.exists ? "unlockVault" : "createFileVault")) return true;
  return requestVault();
}

/**
 * Runs `command`; if it fails because the vault is locked and the dialog
 * that readies it is mounted, asks for it and, once the vault is open, runs
 * the command once more (DEC-041.2). Any other failure, a second one, or a
 * vault left locked goes to `reportFailure`.
 */
export async function runReported(
  command: () => Promise<unknown>,
  queryClient: QueryClient,
  canAsk: CanReadyVault,
): Promise<void> {
  try {
    await command();
  } catch (failure) {
    if (isVaultLocked(failure)) {
      const vault = await queryClient
        .fetchQuery({ ...vaultStatusQuery, staleTime: 0 })
        .catch(() => undefined);
      const dialog = vault?.exists === false ? "createFileVault" : "unlockVault";
      if (vault?.locked && canAsk(dialog) && (await requestVault())) {
        return runReported(command, queryClient, () => false);
      }
    }
    reportFailure(failure, queryClient);
  }
}
