import type { HTMLAttributes, ReactNode } from "react";

import styles from "./Card.module.css";
import { classNames } from "./classNames";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  /** `tinted`: the faint fill of "Credenziale effettiva" and the unsupported box. */
  tinted?: boolean;
  /** `large`: radius 14 instead of 12. */
  large?: boolean;
  /** `warning`: the box "È usata da N nodi" of the delete confirmation. */
  tone?: "plain" | "warning" | "error";
  children: ReactNode;
}

/** An inner box (inventario §4.26). */
export function Card({
  tinted = false,
  large = false,
  tone = "plain",
  className,
  children,
  ...rest
}: CardProps) {
  return (
    <div
      {...rest}
      className={classNames(
        styles.card,
        tinted && styles.tinted,
        large && styles.large,
        styles[tone],
        className,
      )}
    >
      {children}
    </div>
  );
}
