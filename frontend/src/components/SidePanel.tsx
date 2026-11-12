import { type ReactNode, type RefObject, useRef } from "react";
import { createPortal } from "react-dom";

import styles from "./SidePanel.module.css";
import { useModal } from "./useModal";

interface SidePanelProps {
  open: boolean;
  /** Esc and a click on the veil. */
  onClose: () => void;
  /** The accessible name, such as "Modifica connessione SSH". */
  label: string;
  initialFocus?: RefObject<HTMLElement | null>;
  children: ReactNode;
}

/**
 * The panel of the editor (inventario §4.17): it enters from the right over a
 * veil at 40%, as a modal surface. A dialog opened from it stays above it.
 */
export function SidePanel({ open, onClose, label, initialFocus, children }: SidePanelProps) {
  const surface = useRef<HTMLDivElement>(null);
  const keyDown = useModal({ open, onClose, container: surface, initialFocus });
  if (!open) return null;

  return createPortal(
    <div
      className={styles.veil}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div className={styles.column}>
        <div
          ref={surface}
          role="dialog"
          aria-modal="true"
          aria-label={label}
          tabIndex={-1}
          className={styles.panel}
          onKeyDown={keyDown}
        >
          {children}
        </div>
      </div>
    </div>,
    document.body,
  );
}
