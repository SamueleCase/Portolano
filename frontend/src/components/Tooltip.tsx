import {
  cloneElement,
  type FocusEvent,
  type PointerEvent,
  type ReactElement,
  useEffect,
  useId,
  useState,
} from "react";
import { createPortal } from "react-dom";

import { classNames } from "./classNames";
import styles from "./Tooltip.module.css";

/** The tooltip appears after 500 ms (inventario §4.14). */
const TOOLTIP_DELAY_MS = 500;

interface TriggerProps {
  "aria-describedby"?: string | undefined;
  onPointerEnter?: (event: PointerEvent<HTMLElement>) => void;
  onPointerLeave?: (event: PointerEvent<HTMLElement>) => void;
  onFocus?: (event: FocusEvent<HTMLElement>) => void;
  onBlur?: (event: FocusEvent<HTMLElement>) => void;
}

interface TooltipProps {
  /** The text; untrusted strings must already be made visible by the caller. */
  content: string;
  /** A keyboard shortcut in mono, such as "⌘↩". */
  shortcut?: string | undefined;
  /** Wraps on more lines, up to 260 px. */
  multiline?: boolean;
  /** The element described, which must accept focus and pointer events. */
  children: ReactElement<TriggerProps>;
}

interface Position {
  top: number;
  left: number;
}

/**
 * A tooltip on hover and keyboard focus, after 500 ms; Esc hides it. The
 * trigger is described by it (`aria-describedby`).
 */
export function Tooltip({ content, shortcut, multiline = false, children }: TooltipProps) {
  const id = useId();
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  const [position, setPosition] = useState<Position>();

  const hide = () => {
    setAnchor(null);
    setPosition(undefined);
  };

  useEffect(() => {
    if (!anchor) return undefined;
    const timer = window.setTimeout(() => {
      const box = anchor.getBoundingClientRect();
      setPosition({ top: box.bottom, left: box.left + box.width / 2 });
    }, TOOLTIP_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [anchor]);

  useEffect(() => {
    if (!position) return undefined;
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") hide();
    };
    document.addEventListener("keydown", escape);
    return () => document.removeEventListener("keydown", escape);
  }, [position]);

  const trigger = cloneElement(children, {
    "aria-describedby": id,
    onPointerEnter: (event) => {
      children.props.onPointerEnter?.(event);
      setAnchor(event.currentTarget);
    },
    onPointerLeave: (event) => {
      children.props.onPointerLeave?.(event);
      hide();
    },
    onFocus: (event) => {
      children.props.onFocus?.(event);
      setAnchor(event.currentTarget);
    },
    onBlur: (event) => {
      children.props.onBlur?.(event);
      hide();
    },
  });

  return (
    <>
      {trigger}
      {createPortal(
        <span
          id={id}
          role="tooltip"
          hidden={!position}
          className={classNames(styles.tooltip, multiline && styles.multiline)}
          style={position ? { top: position.top, left: position.left } : undefined}
        >
          {content}
          {shortcut && <span className={styles.shortcut}>{shortcut}</span>}
        </span>,
        document.body,
      )}
    </>
  );
}
