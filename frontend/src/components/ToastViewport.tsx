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
 * (ambiguity 17), in a polite live region; errors are alerts.
 */
export function ToastViewport({ toasts, onDismiss }: ToastViewportProps) {
  return createPortal(
    <section className={styles.viewport} aria-label={texts.components.toast.region}>
      <ol className={styles.list} aria-live="polite">
        {toasts.slice(-MAX_VISIBLE_TOASTS).map((toast) => (
          <Toast key={toast.id} toast={toast} onDismiss={onDismiss} />
        ))}
      </ol>
    </section>,
    document.body,
  );
}
