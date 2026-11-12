import { classNames } from "../classNames";
import type { BuiltinIconName } from "./builtinIconNames";
import { builtinIcons } from "./builtinIcons";
import styles from "./Icon.module.css";
import { type IconSize, iconSizeClass } from "./iconSizes";

/** A builtin icon of the document (`Icon::Builtin`), in the current color. Decorative. */
export function BuiltinIcon({ name, size = "tree" }: { name: BuiltinIconName; size?: IconSize }) {
  const Glyph = builtinIcons[name];
  return <Glyph aria-hidden="true" className={classNames(styles.icon, iconSizeClass[size])} />;
}
