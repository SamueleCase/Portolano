import styles from "./Icon.module.css";

/**
 * The sizes an icon takes in the mock (inventario §4.19): badge 13 px,
 * vault 14, list 15, tree 16, tile 20, editor 22, detail 26. The stroke
 * follows the size, as in the mock.
 */
export type IconSize = "badge" | "vault" | "list" | "tree" | "tile" | "editor" | "detail";

/** The class that sizes an icon. */
export const iconSizeClass: Record<IconSize, string | undefined> = {
  badge: styles.badge,
  vault: styles.vault,
  list: styles.list,
  tree: styles.tree,
  tile: styles.tile,
  editor: styles.editor,
  detail: styles.detail,
};
