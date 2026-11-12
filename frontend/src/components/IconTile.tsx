import type { CSSProperties } from "react";

import { classNames } from "./classNames";
import { NodeIcon, type NodeIconChoice } from "./icons/NodeIcon";
import styles from "./IconTile.module.css";
import { nodeColor } from "./nodeColor";
import type { NodeKind } from "./nodeKind";

interface IconTileProps {
  kind: NodeKind;
  icon?: NodeIconChoice | undefined;
  /** The color of the node tints the tile in the detail (ambiguity 15). */
  color?: string | null | undefined;
  /** `detail`: 52 px; `editor`: 44 px with a border. */
  size?: "detail" | "editor";
}

/** The tile with the icon of a node (inventario §4.26). Decorative. */
export function IconTile({ kind, icon, color, size = "detail" }: IconTileProps) {
  const tint = nodeColor(color);
  const style: (CSSProperties & { "--node-color": string }) | undefined = tint
    ? { "--node-color": tint }
    : undefined;
  return (
    <span
      className={classNames(styles.tile, styles[size], tint && styles.tinted)}
      style={style}
      aria-hidden="true"
    >
      <NodeIcon kind={kind} icon={icon} size={size === "detail" ? "detail" : "editor"} />
    </span>
  );
}
