import type { ErrorCode } from "../bindings";
import { texts } from "../i18n/it";
import { IpcFailure, type IpcError } from "./result";

/**
 * How the interface shows an error, chosen from its code (SPEC-UI §4.2):
 * - `field`: under the field the backend names;
 * - `chooseCandidate`: a dialog to choose among the matching nodes;
 * - `confirmDelete`: the confirmation listing who uses the node;
 * - `unlockVault`: the unlock dialog of the file vault;
 * - `reloadBanner` / `missingBanner`: the banners of the document (§5.4);
 * - `firstRun`: the first-run screen, without a document;
 * - `toast`: a persistent error toast.
 */
export type ErrorPresentation =
  | "field"
  | "chooseCandidate"
  | "confirmDelete"
  | "unlockVault"
  | "reloadBanner"
  | "missingBanner"
  | "firstRun"
  | "toast";

/** An error ready to show: where, with which title and message. */
export interface PresentedError {
  presentation: ErrorPresentation;
  title: string;
  message: string;
  /** The field of the editor, for `field`. */
  field?: string;
}

const presentationByCode: Record<ErrorCode, ErrorPresentation> = {
  notFound: "toast",
  ambiguous: "chooseCandidate",
  invalid: "toast",
  referenced: "confirmDelete",
  vaultLocked: "unlockVault",
  vault: "toast",
  modifiedExternally: "reloadBanner",
  documentMissing: "missingBanner",
  noDocument: "firstRun",
  connect: "toast",
  io: "toast",
  internal: "toast",
};

/** How to show `error`; an invalid value with a field goes under the field. */
export function presentError(error: IpcError): PresentedError {
  const title = texts.ipc.errorTitles[error.code];
  if (error.details?.kind === "invalid") {
    return { presentation: "field", title, message: error.message, field: error.details.field };
  }
  return { presentation: presentationByCode[error.code], title, message: error.message };
}

/** How to show any failure of a query or a mutation. */
export function presentFailure(failure: unknown): PresentedError {
  if (failure instanceof IpcFailure) {
    return presentError(failure.ipc);
  }
  return {
    presentation: "toast",
    title: texts.ipc.errorTitles.internal,
    message: texts.ipc.unexpected,
  };
}
