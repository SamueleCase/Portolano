import { useEffect, useEffectEvent, useState } from "react";

import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import styles from "./Toast.module.css";
import { toastDuration, type ToastKind } from "./toastTiming";
import { visibleText } from "./visibleText";

/** A notification (inventario §4.18). */
export interface ToastMessage {
  id: string;
  kind: ToastKind;
  /** May carry names of the document. */
  text: string;
  /** A link on the right, such as "Annulla" or "Riprova". */
  action?: { label: string; onAction: () => void };
}

interface ToastProps {
  toast: ToastMessage;
  onDismiss: (id: string) => void;
}

/**
 * One toast. It leaves by itself after its time, which stops while the
 * pointer or the focus is on it; errors stay until "×".
 */
export function Toast({ toast, onDismiss }: ToastProps) {
  const [held, setHeld] = useState(false);
  const duration = toastDuration[toast.kind];
  const dismiss = useEffectEvent(() => onDismiss(toast.id));

  useEffect(() => {
    if (duration === undefined || held) return undefined;
    const timer = window.setTimeout(dismiss, duration);
    return () => window.clearTimeout(timer);
  }, [duration, held]);

  return (
    <li
      className={classNames(styles.toast, styles[toast.kind])}
      role={toast.kind === "error" ? "alert" : undefined}
      onPointerEnter={() => setHeld(true)}
      onPointerLeave={() => setHeld(false)}
      onFocus={() => setHeld(true)}
      onBlur={() => setHeld(false)}
    >
      <span className={styles.marker} aria-hidden="true" />
      <span className={styles.text}>{visibleText(toast.text)}</span>
      {toast.action && (
        <button
          type="button"
          className={styles.action}
          onClick={() => {
            toast.action?.onAction();
            onDismiss(toast.id);
          }}
        >
          {toast.action.label}
        </button>
      )}
      {toast.kind === "error" && (
        <button
          type="button"
          className={styles.close}
          aria-label={texts.components.toast.close}
          onClick={() => onDismiss(toast.id)}
        >
          ×
        </button>
      )}
    </li>
  );
}
