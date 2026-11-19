import { useLayoutEffect, useRef } from "react";
import { createPortal } from "react-dom";

import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { classNames } from "../classNames";
import tooltipStyles from "../Tooltip.module.css";
import { visibleText } from "../visibleText";
import styles from "./Terminal.module.css";
import { linkParts } from "./terminalSafety";

/** The space kept between the hint and the edges of the window, in pixels. */
const EDGE = 8;

interface LinkHintProps {
  /** The link under the pointer, already openable (`openableLink`). */
  url: string;
  /** Where the pointer is, in the coordinates of the window. */
  x: number;
  y: number;
}

/**
 * The address a link of the terminal would open, under the pointer, with
 * the look of the tooltips of the components (DEC-046.3, DEC-047.3): the
 * host first and whole, then the address with the path shortened in the
 * middle, and a warning if the address carries credentials
 * (`https://intranet.example.com@evil.example/`). It stays inside the
 * window.
 */
export function LinkHint({ url, x, y }: LinkHintProps) {
  const hint = useRef<HTMLSpanElement>(null);
  const { host, address, user } = linkParts(url);

  // Placed once its size is known: centered under the pointer, moved back
  // inside the window, above the pointer if there is no room below.
  useLayoutEffect(() => {
    const element = hint.current;
    if (!element) return;
    const { width, height } = element.getBoundingClientRect();
    const half = width / 2;
    const left = Math.min(Math.max(x, EDGE + half), window.innerWidth - EDGE - half);
    const below = y + height + EDGE <= window.innerHeight;
    element.style.left = `${left}px`;
    element.style.top = `${below ? y : Math.max(EDGE, y - height - 2 * EDGE)}px`;
  }, [x, y, url]);

  return createPortal(
    <span
      ref={hint}
      role="tooltip"
      className={classNames(tooltipStyles.tooltip, tooltipStyles.multiline, styles.linkHint)}
      style={{ top: y, left: x }}
    >
      <strong className={styles.linkHost}>{visibleText(host)}</strong>
      <span className={styles.linkAddress}>{visibleText(address)}</span>
      {user !== null && (
        <span className={styles.linkCredentials}>
          {interpolate(texts.components.terminal.linkCredentials, { user: visibleText(user) })}
        </span>
      )}
    </span>,
    document.body,
  );
}
