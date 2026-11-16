import { useWindowActions } from "../../app/windowActions";
import { ContextMenu, type MenuEntry, type MenuItem } from "../../components/ContextMenu";
import { isConnection } from "../../components/nodeKind";
import { texts } from "../../i18n/it";
import type { NewNodeKind } from "../../state/requests";
import { useWorkspace } from "../../state/workspace";
import { siblingPlacement, type TreeEntry, type TreeIndex } from "./treeModel";

interface NodeContextMenuProps {
  entry: TreeEntry;
  /** The tree and the nodes shown, for "Sposta su" / "Sposta giù". */
  index: TreeIndex;
  visible: ReadonlySet<string> | null;
  position: { x: number; y: number };
  /** "Sposta in…": the picker of the destination folder. */
  onMoveTo: (id: string) => void;
  onClose: () => void;
}

type Action =
  | "connect"
  | "terminal"
  | "command"
  | "edit"
  | "duplicate"
  | "favorite"
  | "move"
  | "up"
  | "down"
  | "delete"
  | `new-${NewNodeKind}`;

const SEPARATOR = { separator: true } as const;

/**
 * The context menu of a node (inventario §4.15; for folders, credentials and
 * unsupported nodes §5.17): what can be done with it, each entry as the
 * button or the shortcut of the same command. A folder also creates in
 * itself (the menu selects the node first, and the new nodes go into the
 * selected folder).
 */
export function NodeContextMenu({
  entry,
  index,
  visible,
  position,
  onMoveTo,
  onClose,
}: NodeContextMenuProps) {
  const actions = useWindowActions();
  const showCommand = useWorkspace((state) => state.showCommand);
  const m = texts.tree.menu;
  const s = texts.app.shortcuts;
  const { id } = entry.node;
  const noEditor = !actions.offers("editor");
  const editorReason = noEditor ? texts.app.reasons.unavailable : undefined;
  const item = (action: Action, label: string, extra: Omit<MenuItem, "id" | "label"> = {}) =>
    ({ id: action, label, ...extra }) satisfies MenuItem;

  const edit = item("edit", m.edit, { shortcut: s.edit, disabled: noEditor, reason: editorReason });
  const duplicate = item("duplicate", m.duplicate, { shortcut: s.duplicate });
  const favorite = item("favorite", entry.node.favorite ? m.unfavorite : m.favorite);
  const move = item("move", m.moveTo);
  // The keyboard alternative to dragging among the siblings (SPEC-UI §5.5).
  const up = item("up", m.moveUp, {
    shortcut: s.moveUp,
    disabled: siblingPlacement(index, id, -1, visible) === undefined,
  });
  const down = item("down", m.moveDown, {
    shortcut: s.moveDown,
    disabled: siblingPlacement(index, id, 1, visible) === undefined,
  });
  const remove = item("delete", m.delete, { shortcut: s.delete, destructive: true });

  const entries: MenuEntry[] = (() => {
    if (isConnection(entry.kind)) {
      return [
        item("connect", m.connect, { shortcut: s.connect }),
        item("terminal", m.openInTerminal),
        item("command", m.showCommand),
        SEPARATOR,
        edit,
        duplicate,
        favorite,
        move,
        up,
        down,
        SEPARATOR,
        remove,
      ];
    }
    if (entry.kind === "folder") {
      return [
        item("new-ssh", m.newSsh, { disabled: noEditor, reason: editorReason }),
        item("new-rdp", m.newRdp, { disabled: noEditor, reason: editorReason }),
        item("new-folder", m.newFolder, { disabled: noEditor, reason: editorReason }),
        item("new-credential", m.newCredential, { disabled: noEditor, reason: editorReason }),
        SEPARATOR,
        edit,
        duplicate,
        favorite,
        move,
        up,
        down,
        SEPARATOR,
        remove,
      ];
    }
    if (entry.kind === "credential") {
      return [edit, duplicate, favorite, move, up, down, SEPARATOR, remove];
    }
    // Unsupported: read-only, it can only be moved or deleted (DEC-041.4).
    return [move, up, down, SEPARATOR, remove];
  })();

  const choose = (action: string) => {
    switch (action) {
      case "connect":
        return actions.connect(id);
      case "terminal":
        return actions.openInTerminal(id);
      case "command":
        return showCommand(id);
      case "edit":
        return actions.edit(id);
      case "duplicate":
        return actions.duplicate(id);
      case "favorite":
        return actions.toggleFavorite(id);
      case "move":
        return onMoveTo(id);
      case "up":
        return actions.moveBy(id, -1, visible);
      case "down":
        return actions.moveBy(id, 1, visible);
      case "delete":
        return actions.remove(id);
      case "new-ssh":
        return actions.create("ssh");
      case "new-rdp":
        return actions.create("rdp");
      case "new-folder":
        return actions.create("folder");
      case "new-credential":
        return actions.create("credential");
    }
  };

  return (
    <ContextMenu
      label={entry.node.name}
      entries={entries}
      position={position}
      onSelect={choose}
      onClose={onClose}
    />
  );
}
