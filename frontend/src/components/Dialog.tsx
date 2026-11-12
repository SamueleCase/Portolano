import { type ReactNode, type RefObject, useId, useRef } from "react";
import { createPortal } from "react-dom";

import { classNames } from "./classNames";
import styles from "./Dialog.module.css";
import { useModal } from "./useModal";
import { visibleText } from "./visibleText";

/** The widths of the dialogs (inventario §4.16). */
export type DialogWidth = "small" | "medium" | "wizard" | "preferences" | "selector" | "report";

interface DialogProps {
  open: boolean;
  /** Esc, a click on the veil and the dialog's own "Annulla". */
  onClose: () => void;
  /** The visible title; without it, `label` names the dialog. */
  title?: string | undefined;
  label?: string | undefined;
  /** The monospaced line under the title, such as the path of a node. */
  subtitle?: string | undefined;
  width?: DialogWidth;
  /** Centered text and the padding of the unlock and first-run dialogs. */
  centered?: boolean;
  /** The buttons at the bottom, right aligned. */
  footer?: ReactNode;
  initialFocus?: RefObject<HTMLElement | null>;
  children?: ReactNode;
}

/**
 * A modal dialog (inventario §4.16): veil at 45% (ambiguity 4), glass of the
 * dialogs, focus trapped inside, Esc and a click on the veil close it.
 */
export function Dialog({
  open,
  onClose,
  title,
  label,
  subtitle,
  width = "small",
  centered = false,
  footer,
  initialFocus,
  children,
}: DialogProps) {
  const surface = useRef<HTMLDivElement>(null);
  const titleId = useId();
  const keyDown = useModal({ open, onClose, container: surface, initialFocus });
  if (!open) return null;

  return createPortal(
    <div
      className={styles.veil}
      onMouseDown={(event) => {
        // Only a click on the veil itself, not one that started inside the dialog.
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        ref={surface}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
        aria-label={title || !label ? undefined : visibleText(label)}
        tabIndex={-1}
        className={classNames(styles.dialog, styles[width], centered && styles.centered)}
        onKeyDown={keyDown}
      >
        {title && (
          <header className={styles.header}>
            <h2 id={titleId} className={styles.title}>
              {visibleText(title)}
            </h2>
            {subtitle && <p className={styles.subtitle}>{visibleText(subtitle)}</p>}
          </header>
        )}
        {children}
        {footer && <footer className={styles.footer}>{footer}</footer>}
      </div>
    </div>,
    document.body,
  );
}
