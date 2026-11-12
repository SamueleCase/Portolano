/**
 * - `success` and `info`: confirmations, gone after 5 s;
 * - `undo`: an action that can be undone ("Eliminato · Annulla"), 10 s;
 * - `error`: stays until closed with "×" (ambiguity 17).
 */
export type ToastKind = "success" | "info" | "undo" | "error";

/** How long a toast stays, in milliseconds; `undefined` = until closed. */
export const toastDuration: Record<ToastKind, number | undefined> = {
  success: 5000,
  info: 5000,
  undo: 10_000,
  error: undefined,
};

/** At most three toasts on screen (ambiguity 17): the newest ones. */
export const MAX_VISIBLE_TOASTS = 3;
