import { Fragment } from "react";

import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import type { NodeKind } from "./nodeKind";
import styles from "./RouteChain.module.css";
import { visibleText } from "./visibleText";

/** A stop of a chain after "tu": a jump, a folder or the destination. */
export interface ChainStop {
  /** A host, a node name or a path: untrusted text. */
  label: string;
  /** The type of the destination, for the color of its dot in the detail. */
  kind?: NodeKind;
}

interface RouteChainProps {
  /**
   * - `route`: the route of the detail, "tu → jumps → host" (inventario §4.21);
   * - `editor`: the chain under the jump list, destination filled;
   * - `inherit`: where an inherited credential comes from, last chip filled.
   */
  variant: "route" | "editor" | "inherit";
  stops: readonly ChainStop[];
  label?: string;
}

/** A chain of chips joined by short lines. */
export function RouteChain({
  variant,
  stops,
  label = texts.components.route.label,
}: RouteChainProps) {
  const chips: ChainStop[] =
    variant === "inherit" ? [...stops] : [{ label: texts.components.route.origin }, ...stops];
  return (
    <ol aria-label={label} className={classNames(styles.chain, styles[variant])}>
      {chips.map((stop, index) => {
        const last = index === chips.length - 1;
        return (
          <Fragment key={`${index}-${stop.label}`}>
            {index > 0 && <li className={styles.connector} aria-hidden="true" />}
            <li className={classNames(styles.chip, last && styles.destination)}>
              {variant === "route" && (
                <span
                  className={classNames(styles.dot, last && stop.kind && styles[stop.kind])}
                  aria-hidden="true"
                />
              )}
              {visibleText(stop.label)}
            </li>
          </Fragment>
        );
      })}
    </ol>
  );
}
