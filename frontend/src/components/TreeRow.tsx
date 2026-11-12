import type { CSSProperties, HTMLAttributes, Ref } from "react";

import { plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import { HighlightedText } from "./HighlightedText";
import { NodeIcon, type NodeIconChoice } from "./icons/NodeIcon";
import { nodeColor } from "./nodeColor";
import type { NodeKind } from "./nodeKind";
import type { SecretState, VaultBackend } from "./SecretBadge";
import { type DotState, StatusDot } from "./StatusDot";
import styles from "./TreeRow.module.css";
import { visibleText } from "./visibleText";

/** What a row shows on the right (inventario §4.19). */
export type TreeRowMeta =
  | { kind: "host"; host: string }
  | { kind: "secret"; state: SecretState; backend: VaultBackend }
  /** A closed folder: all its descendants (ambiguity 21). */
  | { kind: "descendants"; count: number };

interface TreeRowProps extends Omit<HTMLAttributes<HTMLDivElement>, "color"> {
  name: string;
  kind: NodeKind;
  icon?: NodeIconChoice | undefined;
  /** 0 for the root level. */
  depth: number;
  /** Only folders: open or closed. */
  expanded?: boolean | undefined;
  selected?: boolean;
  favorite?: boolean;
  /** The color of the node, a dot before the name (ambiguity 15). */
  color?: string | null | undefined;
  meta?: TreeRowMeta | undefined;
  session?: DotState;
  /** The part of the name that matches the search. */
  highlight?: string | undefined;
  /** Drag and drop (inventario §3.4): the line above, or the folder that receives. */
  dropTarget?: "before" | "inside" | undefined;
  dragging?: boolean;
  ref?: Ref<HTMLDivElement>;
}

const secretMeta: Record<SecretState, string> = {
  present: "",
  absent: texts.components.tree.meta.absent,
  staged: texts.components.tree.meta.staged,
  locked: texts.components.tree.meta.locked,
};

function metaText(meta: TreeRowMeta | undefined, kind: NodeKind, expanded: boolean | undefined) {
  if (kind === "unsupported") {
    return { text: texts.components.tree.meta.unsupported, tone: styles.metaUnsupported };
  }
  switch (meta?.kind) {
    case "host":
      return { text: visibleText(meta.host), tone: undefined };
    case "secret":
      return meta.state === "present"
        ? { text: texts.components.tree.meta.present[meta.backend], tone: styles.metaOk }
        : {
            text: secretMeta[meta.state],
            tone: meta.state === "absent" ? styles.metaWarn : undefined,
          };
    case "descendants":
      return expanded
        ? undefined
        : { text: plural(meta.count, texts.components.tree.descendants), tone: undefined };
    case undefined:
      return undefined;
  }
}

/**
 * A row of the node tree in every state of inventario §4.19, with the roles
 * of a `treeitem`. The tree itself (Headless Tree, UI-WP4) passes its own
 * props and handlers through.
 */
export function TreeRow({
  name,
  kind,
  icon,
  depth,
  expanded,
  selected = false,
  favorite = false,
  color,
  meta,
  session = "idle",
  highlight,
  dropTarget,
  dragging = false,
  className,
  style,
  ...rest
}: TreeRowProps) {
  const folder = kind === "folder";
  const shownMeta = metaText(meta, kind, expanded);
  const dot = nodeColor(color);
  // The indentation of inventario §4.19 (8 px + 18 px per level) lives in the stylesheet.
  const rowStyle: CSSProperties & { "--tree-depth": number } = { ...style, "--tree-depth": depth };
  return (
    <div
      role="treeitem"
      aria-level={depth + 1}
      aria-expanded={folder ? Boolean(expanded) : undefined}
      aria-selected={selected}
      {...rest}
      className={classNames(
        styles.row,
        selected && styles.selected,
        dropTarget === "before" && styles.dropBefore,
        dropTarget === "inside" && styles.dropInside,
        dragging && styles.dragging,
        kind === "unsupported" && styles.unsupported,
        className,
      )}
      style={rowStyle}
    >
      <span className={styles.chevron} aria-hidden="true">
        {folder ? (expanded ? "▾" : "▸") : ""}
      </span>
      <NodeIcon kind={kind} icon={icon} />
      {dot && <span className={styles.color} style={{ background: dot }} aria-hidden="true" />}
      <span className={styles.name}>
        <HighlightedText text={name} match={highlight} />
      </span>
      {favorite && (
        <span className={styles.star} role="img" aria-label={texts.components.tree.favorite}>
          ★
        </span>
      )}
      {shownMeta && (
        <span className={classNames(styles.meta, shownMeta.tone)}>{shownMeta.text}</span>
      )}
      <StatusDot state={session} />
    </div>
  );
}
