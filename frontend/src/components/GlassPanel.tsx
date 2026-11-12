import type { HTMLAttributes, ReactNode } from "react";

import { classNames } from "./classNames";
import styles from "./GlassPanel.module.css";

interface GlassPanelProps extends HTMLAttributes<HTMLElement> {
  /** `panel`: the 18 px glass of the window; `inner`: the 14 px session header and tab bar. */
  variant?: "panel" | "inner";
  as?: "div" | "section" | "aside" | "header" | "main";
  children: ReactNode;
}

/** A surface of panel glass (inventario §2.11, §4.26). */
export function GlassPanel({
  variant = "panel",
  as: Element = "div",
  className,
  children,
  ...rest
}: GlassPanelProps) {
  return (
    <Element {...rest} className={classNames(styles.glass, styles[variant], className)}>
      {children}
    </Element>
  );
}
