import { createPortal } from "react-dom";

import { texts } from "../i18n/it";
import { Toast, type ToastMessage } from "./Toast";
import { MAX_VISIBLE_TOASTS } from "./toastTiming";
import styles from "./Toast.module.css";
import { visibleText } from "./visibleText";

interface ToastViewportProps {
  toasts: readonly ToastMessage[];
  onDismiss: (id: string) => void;
}

/**
 * The toasts at the bottom right, 360 px wide, the newest three in the order
 * they came, the newest by the edge (ambiguity 17). The list itself is not a
 * live region: each toast is announced once, by a hidden copy of its text in
 * a polite region (confirmations) or an assertive one (errors).
 */
export function ToastViewport({ toasts, onDismiss }: ToastViewportProps) {
  const shown = toasts.slice(-MAX_VISIBLE_TOASTS);
  const announce = (errors: boolean) =>
    shown
      .filter((toast) => (toast.kind === "error") === errors)
      .map((toast) => <p key={toast.id}>{visibleText(toast.text)}</p>);
  return createPortal(
    <section className={styles.viewport} aria-label={texts.components.toast.region}>
      <ol className={styles.list}>
        {shown.map((toast) => (
          <Toast key={toast.id} toast={toast} onDismiss={onDismiss} />
        ))}
      </ol>
      <div className={styles.announcement} aria-live="polite">
        {announce(false)}
      </div>
      <div className={styles.announcement} aria-live="assertive">
        {announce(true)}
      </div>
    </section>,
    document.body,
  );
}
