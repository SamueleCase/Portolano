import { classNames } from "../classNames";
import type { NodeKind } from "../nodeKind";
import { BuiltinIcon } from "./BuiltinIcon";
import { isBuiltinIconName } from "./builtinIconNames";
import { customIconUrl } from "./customIcon";
import styles from "./Icon.module.css";
import { type IconSize, iconSizeClass } from "./iconSizes";
import { NodeTypeIcon } from "./NodeTypeIcon";

/** The icon a node carries in the document. */
export type NodeIconChoice =
  { source: "builtin"; name: string } | { source: "custom"; fileName: string };

interface NodeIconProps {
  kind: NodeKind;
  icon?: NodeIconChoice | undefined;
  size?: IconSize;
}

/**
 * The icon of a node: its builtin or custom icon, else the icon of its type
 * (also for names the set does not know). A custom icon is only ever an
 * `<img>` from `portolano-icon://` (SPEC-UI §1.4). Decorative.
 */
export function NodeIcon({ kind, icon, size = "tree" }: NodeIconProps) {
  if (icon?.source === "builtin" && isBuiltinIconName(icon.name)) {
    return (
      <span className={classNames(styles[kind], styles.holder)}>
        <BuiltinIcon name={icon.name} size={size} />
      </span>
    );
  }
  const url = icon?.source === "custom" ? customIconUrl(icon.fileName) : undefined;
  if (url) {
    return (
      <img
        src={url}
        alt=""
        draggable={false}
        className={classNames(styles.image, iconSizeClass[size])}
      />
    );
  }
  return <NodeTypeIcon kind={kind} size={size} />;
}
