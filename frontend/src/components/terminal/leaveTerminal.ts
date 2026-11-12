import { focusableIn } from "../focusable";

/**
 * Moves the focus to the control after (`direction` 1) or before (-1) the
 * terminal. Tab belongs to the shell inside the terminal, so Ctrl+Tab is the
 * way out (WCAG 2.1.2, no keyboard trap).
 */
export function leaveTerminal(terminal: HTMLElement, direction: 1 | -1): void {
  const outside = focusableIn(terminal.ownerDocument.body).filter(
    (element) => !terminal.contains(element),
  );
  const after = outside.find(
    (element) => terminal.compareDocumentPosition(element) & Node.DOCUMENT_POSITION_FOLLOWING,
  );
  const before = outside.findLast(
    (element) => terminal.compareDocumentPosition(element) & Node.DOCUMENT_POSITION_PRECEDING,
  );
  (direction === 1 ? after : before)?.focus();
}
