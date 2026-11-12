import { type KeyboardEvent, type RefObject, useEffect } from "react";

import { useBackgroundDialog } from "../design/background/backgroundControl";
import { focusableIn } from "./focusable";

interface ModalOptions {
  open: boolean;
  onClose: () => void;
  container: RefObject<HTMLElement | null>;
  /** The element focused on opening; else the first field, else the first control. */
  initialFocus?: RefObject<HTMLElement | null> | undefined;
}

function firstToFocus(container: HTMLElement): HTMLElement | undefined {
  const marked = container.querySelector<HTMLElement>("[data-autofocus]");
  if (marked) return marked;
  const focusable = focusableIn(container);
  return (
    focusable.find((element) => element.matches("input, textarea, [role='combobox']")) ??
    focusable[0]
  );
}

/**
 * The behavior of a modal surface (inventario §4.16, proposal): the focus
 * moves in on opening, Tab cycles inside, Esc closes, and the focus goes back
 * where it was on closing. The background slows down behind it.
 * Returns the key handler to put on the surface.
 */
export function useModal({ open, onClose, container, initialFocus }: ModalOptions) {
  useBackgroundDialog(open);

  useEffect(() => {
    if (!open) return undefined;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const surface = container.current;
    if (surface) {
      (initialFocus?.current ?? firstToFocus(surface) ?? surface).focus();
    }
    return () => {
      previous?.focus();
    };
  }, [open, container, initialFocus]);

  return (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      // Only the topmost modal closes: the event stops here.
      event.stopPropagation();
      onClose();
      return;
    }
    const surface = container.current;
    if (event.key !== "Tab" || !surface) return;
    const focusable = focusableIn(surface);
    const first = focusable[0];
    const last = focusable.at(-1);
    if (!first || !last) {
      event.preventDefault();
      return;
    }
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };
}
