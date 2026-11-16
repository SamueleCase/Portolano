import {
  dragAndDropFeature,
  type DragTarget,
  type ItemInstance,
  syncDataLoaderFeature,
} from "@headless-tree/core";
import { useTree } from "@headless-tree/react";
import { type MouseEvent, useEffect, useMemo, useRef } from "react";

import type { PlacementInput, SecretsBadge } from "../../bindings";
import { isConnection } from "../../components/nodeKind";
import type { SecretState, VaultBackend } from "../../components/SecretBadge";
import type { DotState } from "../../components/StatusDot";
import { type TreeRowMeta, TreeRow } from "../../components/TreeRow";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { useWorkspace } from "../../state/workspace";
import { sessionDot } from "../sessions/sessionState";
import { useSessionsStore } from "../sessions/store";
import { dropPlacement } from "./dropPlacement";
import { rowLabel } from "./rowLabel";
import styles from "./Sidebar.module.css";
import { iconChoice, summaryHost, type TreeEntry, type TreeIndex } from "./treeModel";
import { type TreeKeyActions, useTreeKeyboard } from "./useTreeKeyboard";

/** The id of the invisible root of Headless Tree: the root of the document. */
const ROOT = "root";

/** The indentation of a level (inventario §4.19: 18 px), for the drop line. */
const INDENT = 18;

interface NodeTreeProps {
  index: TreeIndex;
  /** The nodes shown; `null`: all of them. */
  visible: ReadonlySet<string> | null;
  /** The text searched, highlighted in the names. */
  query: string;
  backend: VaultBackend;
  /** A node to connect to (double click, ⌘↩). */
  onConnect: (id: string) => void;
  onMove: (id: string, destination: string | null, placement: PlacementInput) => void;
  /** Alt+↑ / Alt+↓ on a row. */
  onMoveBy: (id: string, step: -1 | 1) => void;
  /** The context menu of node `id`, at `position` in the viewport. */
  onContextMenu: (id: string, position: { x: number; y: number }) => void;
}

const secretStates: Record<SecretsBadge, SecretState | undefined> = {
  none: undefined,
  present: "present",
  absent: "absent",
  vaultLocked: "locked",
};

/** What a row shows on the right (inventario §4.19). */
function metaOf(entry: TreeEntry, backend: VaultBackend): TreeRowMeta | undefined {
  const { node } = entry;
  if (entry.kind === "folder") return { kind: "descendants", count: node.descendants };
  // The summary is `user@host:port`: the row shows the host.
  if (isConnection(entry.kind)) return { kind: "host", host: summaryHost(node.summary) };
  const state = secretStates[node.secrets];
  return entry.kind === "credential" && state !== undefined
    ? { kind: "secret", state, backend }
    : undefined;
}

/**
 * The tree of the document (inventario §4.19, §5.1, SPEC-UI §5.5) on Headless
 * Tree, with the keys of `useTreeKeyboard`; a node is moved by dragging it,
 * with Alt+↑/↓ or with "Sposta in…". One row always takes the Tab key: the
 * focused one, else the selection, else the first.
 */
export function NodeTree(props: NodeTreeProps) {
  const { index, visible, query, backend, onConnect, onMove, onMoveBy, onContextMenu } = props;
  const selectedId = useWorkspace((state) => state.selectedId);
  const select = useWorkspace((state) => state.select);
  const expandedState = useWorkspace((state) => state.expanded);
  const setExpanded = useWorkspace((state) => state.setExpanded);
  const revealRequests = useWorkspace((state) => state.revealRequests);
  const sessions = useSessionsStore((state) => state.sessions);
  const revealed = useRef(revealRequests);

  const rootFolders = useMemo(
    () => index.rootIds.filter((id) => index.entries.get(id)?.kind === "folder"),
    [index],
  );
  const userExpanded = expandedState ?? rootFolders;
  // While searching or filtering every folder shown is open (inventario §5.1).
  const expandedItems = useMemo(
    () =>
      visible === null
        ? [...userExpanded]
        : [...visible].filter((id) => index.entries.get(id)?.kind === "folder"),
    [visible, userExpanded, index],
  );
  const dots = useMemo(() => {
    const byNode = new Map<string, DotState>();
    for (const session of sessions) byNode.set(session.nodeId, sessionDot(session.state));
    return byNode;
  }, [sessions]);

  const childrenOf = (id: string): string[] => {
    const ids =
      id === ROOT ? index.rootIds : (index.entries.get(id)?.node.children ?? []).map((c) => c.id);
    return visible === null ? [...ids] : ids.filter((child) => visible.has(child));
  };

  const drop = (items: ItemInstance<string>[], target: DragTarget<string>) => {
    const moved = items[0]?.getId();
    if (moved === undefined) return;
    const folder = target.item.getId();
    const placement: PlacementInput =
      "childIndex" in target
        ? dropPlacement(childrenOf(folder), target.childIndex, moved)
        : { kind: "end" };
    onMove(moved, folder === ROOT ? null : folder, placement);
  };

  const tree = useTree<string>({
    rootItemId: ROOT,
    getItemName: (item) => visibleText(index.entries.get(item.getId())?.node.name ?? ""),
    isItemFolder: (item) => index.entries.get(item.getId())?.kind === "folder",
    dataLoader: { getItem: (id) => id, getChildren: childrenOf },
    state: { expandedItems },
    setExpandedItems: (update) => {
      if (visible !== null) return;
      setExpanded(typeof update === "function" ? update(expandedItems) : update);
    },
    onPrimaryAction: (item) => select(item.getId()),
    indent: INDENT,
    canReorder: true,
    onDrop: drop,
    features: [syncDataLoaderFeature, dragAndDropFeature],
  });

  const openMenu = (id: string, position: { x: number; y: number }) => {
    select(id);
    onContextMenu(id, position);
  };
  const keys: TreeKeyActions = { select, connect: onConnect, openMenu, moveBy: onMoveBy };
  const keyDown = useTreeKeyboard(tree, index, keys, ROOT);

  // The data comes from the backend: the tree is rebuilt when it changes.
  // Then one row must keep the Tab key, and a node just revealed (a copy, a
  // node created, "Vai al nodo") takes the focus of the tree and shows.
  useEffect(() => {
    tree.rebuildTree();
    const items = tree.getItems();
    const selected = items.find((item) => item.getId() === selectedId);
    if (revealed.current !== revealRequests && selected) {
      revealed.current = revealRequests;
      selected.setFocused();
      // jsdom has no `scrollIntoView`: only the browsers scroll.
      selected.getElement()?.scrollIntoView?.({ block: "nearest" });
    } else if (!items.some((item) => item.isFocused())) {
      (selected ?? items[0])?.setFocused();
    }
  }, [tree, index, visible, expandedItems, selectedId, revealRequests]);

  // The first opening of the document opens its folders at the root.
  useEffect(() => {
    if (expandedState === null) setExpanded(rootFolders);
  }, [expandedState, rootFolders, setExpanded]);

  const dragged = tree.getState().dnd?.draggedItems ?? [];

  return (
    <div {...tree.getContainerProps(texts.tree.label)} className={styles.tree} onKeyDown={keyDown}>
      {tree.getItems().map((item) => {
        const id = item.getId();
        const entry = index.entries.get(id);
        if (!entry) return null;
        const meta = metaOf(entry, backend);
        const session = dots.get(id) ?? "idle";
        const expanded = entry.kind === "folder" ? item.isExpanded() : undefined;
        return (
          <TreeRow
            key={id}
            {...item.getProps()}
            aria-selected={selectedId === id}
            aria-label={rowLabel(entry, meta, session, expanded ?? false)}
            name={entry.node.name}
            kind={entry.kind}
            icon={iconChoice(entry.node.icon)}
            depth={item.getItemMeta().level}
            expanded={expanded}
            selected={selectedId === id}
            favorite={entry.node.favorite}
            color={entry.node.color}
            meta={meta}
            session={session}
            highlight={query === "" ? undefined : query}
            dropTarget={
              item.isDragTargetAbove()
                ? "before"
                : entry.kind === "folder" && item.isUnorderedDragTarget()
                  ? "inside"
                  : undefined
            }
            dragging={dragged.some((other) => other.getId() === id)}
            // Focus from outside (Tab, a click elsewhere) is the focus of the tree too.
            onFocus={() => {
              if (!item.isFocused()) item.setFocused();
            }}
            onDoubleClick={() => {
              if (isConnection(entry.kind)) onConnect(id);
            }}
            onContextMenu={(event: MouseEvent) => {
              event.preventDefault();
              openMenu(id, { x: event.clientX, y: event.clientY });
            }}
          />
        );
      })}
    </div>
  );
}
