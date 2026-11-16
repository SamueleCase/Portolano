import {
  dragAndDropFeature,
  type DragTarget,
  type ItemInstance,
  syncDataLoaderFeature,
} from "@headless-tree/core";
import { useTree } from "@headless-tree/react";
import { type KeyboardEvent, type MouseEvent, useEffect, useMemo, useRef } from "react";

import type { PlacementInput, SecretsBadge } from "../../bindings";
import { isConnection } from "../../components/nodeKind";
import type { SecretState, VaultBackend } from "../../components/SecretBadge";
import type { DotState } from "../../components/StatusDot";
import { type TreeRowMeta, TreeRow } from "../../components/TreeRow";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { useWorkspace } from "../../state/workspace";
import { addressHost } from "../details/route";
import { sessionDot } from "../sessions/sessionState";
import { useSessionsStore } from "../sessions/store";
import { dropPlacement } from "./dropPlacement";
import { iconChoice, type TreeEntry, type TreeIndex } from "./treeModel";
import styles from "./Sidebar.module.css";
import { TYPEAHEAD_RESET_MS, typeaheadMatch } from "./typeahead";

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
  // The summary is `user@host:port`: the row shows the host (inventario §4.19).
  if (isConnection(entry.kind)) return { kind: "host", host: addressHost(node.summary) };
  const state = secretStates[node.secrets];
  return entry.kind === "credential" && state !== undefined
    ? { kind: "secret", state, backend }
    : undefined;
}

/**
 * The tree of the document (inventario §4.19, §5.1, SPEC-UI §5.5) on Headless
 * Tree: the arrows, Home and End move, → and ← open and close, Enter shows
 * the detail, ⌘↩ connects, letters jump to a name, Maiusc+F10 opens the
 * context menu; a node is moved by dragging it (or with "Sposta in…").
 * The keys are handled here rather than by the hotkeys of Headless Tree,
 * which track the pressed keys and lose the release of a key held with ⌘
 * (macOS sends no `keyup` for it).
 */
export function NodeTree({
  index,
  visible,
  query,
  backend,
  onConnect,
  onMove,
  onContextMenu,
}: NodeTreeProps) {
  const selectedId = useWorkspace((state) => state.selectedId);
  const select = useWorkspace((state) => state.select);
  const expandedState = useWorkspace((state) => state.expanded);
  const setExpanded = useWorkspace((state) => state.setExpanded);
  const sessions = useSessionsStore((state) => state.sessions);
  const typed = useRef({ text: "", at: 0 });

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
    onDrop: (items, target) => drop(items, target),
    features: [syncDataLoaderFeature, dragAndDropFeature],
  });

  const drop = (items: ItemInstance<string>[], target: DragTarget<string>) => {
    const moved = items[0]?.getId();
    if (moved === undefined) return;
    const folder = target.item.getId();
    const destination = folder === ROOT ? null : folder;
    const placement: PlacementInput =
      "childIndex" in target
        ? dropPlacement(childrenOf(folder), target.childIndex, moved)
        : { kind: "end" };
    onMove(moved, destination, placement);
  };

  // The data comes from the backend: the tree is rebuilt when it changes.
  useEffect(() => {
    tree.rebuildTree();
  }, [tree, index, visible, expandedItems]);

  // The first opening of the document opens its folders at the root.
  useEffect(() => {
    if (expandedState === null) setExpanded(rootFolders);
  }, [expandedState, rootFolders, setExpanded]);

  const focusItem = (item: ItemInstance<string> | undefined) => {
    if (!item) return;
    item.setFocused();
    // At once, not with `updateDomFocus` (a timer): a late focus of an older
    // row would take the focus of the tree back to it (`onFocus` below).
    item.getElement()?.focus();
  };

  const keyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    // The rows as the tree has them now: a key right after another (→ then ↓)
    // comes before React has drawn the rows the first one opened.
    const items = tree.getItems();
    const focused = tree.getFocusedItem();
    const position = items.findIndex((item) => item.getId() === focused.getId());
    const command = event.metaKey || event.ctrlKey;
    const handled = (() => {
      if (event.key === "Enter") {
        if (command) {
          const entry = index.entries.get(focused.getId());
          if (entry && isConnection(entry.kind)) onConnect(focused.getId());
        } else {
          select(focused.getId());
        }
        return true;
      }
      if (command || event.altKey) return false;
      switch (event.key) {
        case "ArrowDown":
          focusItem(items[position + 1]);
          return true;
        case "ArrowUp":
          focusItem(items[Math.max(0, position - 1)]);
          return true;
        case "Home":
          focusItem(items[0]);
          return true;
        case "End":
          focusItem(items.at(-1));
          return true;
        case "ArrowRight":
          if (focused.isFolder() && !focused.isExpanded()) focused.expand();
          else if (focused.isFolder()) focusItem(items[position + 1]);
          return true;
        case "ArrowLeft":
          if (focused.isFolder() && focused.isExpanded()) focused.collapse();
          else if (focused.getParent()?.getId() !== ROOT) focusItem(focused.getParent());
          return true;
        case "ContextMenu":
          openMenuAt(focused);
          return true;
        case "F10":
          if (!event.shiftKey) return false;
          openMenuAt(focused);
          return true;
        default:
          return (
            event.key.length === 1 &&
            event.key !== " " &&
            typeahead(event.key, position, event.timeStamp)
          );
      }
    })();
    if (handled) {
      // ⌘↩ of the tree is not also the ⌘↩ of the window.
      event.preventDefault();
      event.stopPropagation();
    }
  };

  const typeahead = (letter: string, position: number, now: number): boolean => {
    const items = tree.getItems();
    const text = now - typed.current.at > TYPEAHEAD_RESET_MS ? letter : typed.current.text + letter;
    typed.current = { text, at: now };
    const match = typeaheadMatch(
      items.map((item) => item.getItemName()),
      Math.max(position, 0),
      text,
    );
    focusItem(match === undefined ? undefined : items[match]);
    return true;
  };

  const openMenuAt = (item: ItemInstance<string>) => {
    const element = item.getElement();
    const box = element?.getBoundingClientRect();
    select(item.getId());
    onContextMenu(item.getId(), { x: box?.left ?? 0, y: box?.bottom ?? 0 });
  };

  const dragged = tree.getState().dnd?.draggedItems ?? [];

  return (
    <div {...tree.getContainerProps(texts.tree.label)} className={styles.tree} onKeyDown={keyDown}>
      {tree.getItems().map((item) => {
        const id = item.getId();
        const entry = index.entries.get(id);
        if (!entry) return null;
        const props = item.getProps();
        return (
          <TreeRow
            key={id}
            {...props}
            aria-selected={selectedId === id}
            name={entry.node.name}
            kind={entry.kind}
            icon={iconChoice(entry.node.icon)}
            depth={item.getItemMeta().level}
            expanded={entry.kind === "folder" ? item.isExpanded() : undefined}
            selected={selectedId === id}
            favorite={entry.node.favorite}
            color={entry.node.color}
            meta={metaOf(entry, backend)}
            session={dots.get(id) ?? "idle"}
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
              select(id);
              onContextMenu(id, { x: event.clientX, y: event.clientY });
            }}
          />
        );
      })}
    </div>
  );
}
