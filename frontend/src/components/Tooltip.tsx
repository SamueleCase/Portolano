import {
  cloneElement,
  type FocusEvent,
  type PointerEvent,
  type ReactElement,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

import { TOOLTIP_DELAY_MS } from "../design/tokens";
import { classNames } from "./classNames";
import styles from "./Tooltip.module.css";
import { placeTooltip, type TooltipPlacement } from "./tooltipPlacement";
import { visibleText } from "./visibleText";

interface TriggerProps {
  "aria-describedby"?: string | undefined;
  onPointerEnter?: (event: PointerEvent<HTMLElement>) => void;
  onPointerLeave?: (event: PointerEvent<HTMLElement>) => void;
  onFocus?: (event: FocusEvent<HTMLElement>) => void;
  onBlur?: (event: FocusEvent<HTMLElement>) => void;
}

interface TooltipProps {
  /** The text, possibly from the document: it is made visible here. */
  content: string;
  /** A keyboard shortcut in mono, such as "⌘↩". */
  shortcut?: string | undefined;
  /** Wraps on more lines, up to 260 px. */
  multiline?: boolean;
  /** The element described, which must accept focus and pointer events. */
  children: ReactElement<TriggerProps>;
}

/** The style of a tooltip at `placement`. */
function placementStyle(placement: TooltipPlacement) {
  return placement.above
    ? { left: placement.left, bottom: placement.bottom }
    : { left: placement.left, top: placement.top };
}

/**
 * A tooltip on hover and keyboard focus, after 500 ms; Esc hides it. The
 * trigger is described by it (`aria-describedby`). It opens under the
 * trigger, above it when the window has no room below, and stays inside the
 * window sideways.
 */
export function Tooltip({ content, shortcut, multiline = false, children }: TooltipProps) {
  const id = useId();
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  // The box of the trigger once the delay is over; then where the tooltip
  // goes, once its size is known.
  const [triggerBox, setTriggerBox] = useState<DOMRect>();
  const [placement, setPlacement] = useState<TooltipPlacement>();
  const tooltip = useRef<HTMLSpanElement>(null);

  const hide = () => {
    setAnchor(null);
    setTriggerBox(undefined);
    setPlacement(undefined);
  };

  useEffect(() => {
    if (!anchor) return undefined;
    const timer = window.setTimeout(
      () => setTriggerBox(anchor.getBoundingClientRect()),
      TOOLTIP_DELAY_MS,
    );
    return () => window.clearTimeout(timer);
  }, [anchor]);

  // Measured before the paint, laid out under the trigger and unseen: the
  // gap is its top margin there.
  useLayoutEffect(() => {
    const element = tooltip.current;
    if (!triggerBox || !element) return;
    const { width, height } = element.getBoundingClientRect();
    const gap = Number.parseFloat(getComputedStyle(element).marginTop) || 0;
    setPlacement(
      placeTooltip(triggerBox, { width, height }, gap, {
        width: window.innerWidth,
        height: window.innerHeight,
      }),
    );
  }, [triggerBox]);

  useEffect(() => {
    if (!triggerBox) return undefined;
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") hide();
    };
    document.addEventListener("keydown", escape);
    return () => document.removeEventListener("keydown", escape);
  }, [triggerBox]);

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
          ref={tooltip}
          id={id}
          role="tooltip"
          hidden={!triggerBox}
          className={classNames(
            styles.tooltip,
            multiline && styles.multiline,
            placement?.above && styles.above,
          )}
          style={
            placement
              ? placementStyle(placement)
              : triggerBox && {
                  top: triggerBox.bottom,
                  left: triggerBox.left,
                  visibility: "hidden",
                }
          }
        >
          {visibleText(content)}
          {shortcut && <span className={styles.shortcut}>{shortcut}</span>}
        </span>,
        document.body,
      )}
    </>
  );
}
