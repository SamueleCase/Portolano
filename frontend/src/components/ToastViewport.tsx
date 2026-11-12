import { createPortal } from "react-dom";

import { texts } from "../i18n/it";
import { Toast, type ToastMessage } from "./Toast";
import { MAX_VISIBLE_TOASTS } from "./toastTiming";
import styles from "./Toast.module.css";

interface ToastViewportProps {
  toasts: readonly ToastMessage[];
  onDismiss: (id: string) => void;
}

/**
 * The toasts at the bottom right, 360 px wide, the newest three
 * (ambiguity 17). Each toast is announced once: confirmations from a polite
 * live region, errors from an assertive one.
 */
export function ToastViewport({ toasts, onDismiss }: ToastViewportProps) {
  const shown = toasts.slice(-MAX_VISIBLE_TOASTS);
  const list = (kind: "polite" | "assertive") => (
    <ol className={styles.list} aria-live={kind}>
      {shown
        .filter((toast) => (toast.kind === "error") === (kind === "assertive"))
        .map((toast) => (
          <Toast key={toast.id} toast={toast} onDismiss={onDismiss} />
        ))}
    </ol>
  );
  return createPortal(
    <section className={styles.viewport} aria-label={texts.components.toast.region}>
      {list("polite")}
      {list("assertive")}
    </section>,
    document.body,
  );
}
