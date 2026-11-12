import { classNames } from "../classNames";
import type { NodeKind } from "../nodeKind";
import styles from "./Icon.module.css";
import { type IconSize, iconSizeClass } from "./iconSizes";

/** The paths of the node type icons (inventario §4.19, `viewBox 0 0 16 16`). */
const paths: Record<NodeKind, string> = {
  folder: "M2 4.5h4l1.5 1.5H14v6.5H2z",
  ssh: "M3 5l3 3-3 3M8 11h5",
  rdp: "M2 3.5h12v7H2zM6 13h4",
  credential: "M7 8a2 2 0 1 0-4 0 2 2 0 0 0 4 0zM7 8h6M11 8v2",
  unsupported: "M3.5 3.5l9 9M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z",
};

interface NodeTypeIconProps {
  kind: NodeKind;
  size?: IconSize;
  /** Draws the icon in the current color instead of the color of the type. */
  plain?: boolean;
}

/** The icon of a node type, in the color of the type. Decorative. */
export function NodeTypeIcon({ kind, size = "tree", plain = false }: NodeTypeIconProps) {
  return (
    <svg
      viewBox="0 0 16 16"
      aria-hidden="true"
      className={classNames(styles.icon, iconSizeClass[size], !plain && styles[kind])}
    >
      <path d={paths[kind]} />
    </svg>
  );
}
