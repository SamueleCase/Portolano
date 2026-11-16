import type { MenuCommand } from "../bindings";
import { isConnection, type NodeKind } from "../components/nodeKind";
import { DETAIL_TAB, useWorkspace } from "../state/workspace";
import type { WindowActions } from "./windowActions";

/** What the native menu (§4.9) and the shortcuts ask: the menu items, and ⌘1…⌘9. */
export type WindowCommand = MenuCommand | { kind: "showTab"; index: number };

/** What a command acts on. */
export interface CommandContext {
  /** The selected node, if any. */
  selection: { id: string; kind: NodeKind } | undefined;
  /** The tabs in their order, by id. */
  tabs: readonly string[];
  /** A dialog or the editor is open: the window behind waits (SPEC-UI §5.5). */
  modalOpen: boolean;
}

/**
 * Performs `command` with `actions`, the same way for the menu and for the
 * shortcuts (SPEC-UI §4.9, §5.5). Nothing happens behind an open dialog, and
 * a node command without a fitting selection does nothing.
 */
export function runWindowCommand(
  command: WindowCommand,
  actions: WindowActions,
  { selection, tabs, modalOpen }: CommandContext,
): void {
  if (modalOpen) return;
  const workspace = useWorkspace.getState();
  switch (command.kind) {
    case "preferences":
      return actions.openPreferences();
    case "newDocument":
      return actions.newDocument();
    case "openDocument":
      return actions.openDocument();
    case "openRecent":
      return actions.openRecent(command.path);
    case "import":
      return actions.openImport();
    case "closeTab":
      if (workspace.activeTab !== DETAIL_TAB) actions.closeTab(workspace.activeTab);
      return;
    case "newConnection":
      // The protocol comes from the menu of "+ Connessione" (ambiguity 10).
      if (actions.offers("editor")) workspace.setConnectionMenu(true);
      else actions.create("ssh");
      return;
    case "newFolder":
      return actions.create("folder");
    case "newCredential":
      return actions.create("credential");
    case "connect":
      if (selection && isConnection(selection.kind)) actions.connect(selection.id);
      return;
    case "edit":
      if (selection && selection.kind !== "unsupported") actions.edit(selection.id);
      return;
    case "duplicate":
      if (selection) actions.duplicate(selection.id);
      return;
    case "delete":
      if (selection) actions.remove(selection.id);
      return;
    case "search":
      return workspace.requestSearch();
    case "validate":
      return actions.openValidation();
    case "densityNormal":
      return actions.setDensity("normal");
    case "densityCompact":
      return actions.setDensity("compact");
    case "showTab": {
      const tab = tabs[command.index];
      if (tab !== undefined) workspace.showTab(tab);
      return;
    }
  }
}

/** The keys of the shortcuts, as `KeyboardEvent` has them. */
export type ShortcutKey = Pick<
  KeyboardEvent,
  "code" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey" | "target"
>;

/** Whether `target` takes text, where ⌘⌫ deletes text and not a node. */
function editsText(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target.isContentEditable)
  );
}

/** The shortcuts with ⌘ alone, by key code. */
const PLAIN: Readonly<Record<string, WindowCommand>> = {
  Enter: { kind: "connect" },
  KeyE: { kind: "edit" },
  KeyD: { kind: "duplicate" },
  KeyN: { kind: "newConnection" },
  KeyF: { kind: "search" },
  Comma: { kind: "preferences" },
  KeyO: { kind: "openDocument" },
  KeyI: { kind: "import" },
  KeyW: { kind: "closeTab" },
};

/** The shortcuts with ⇧⌘, by key code. */
const SHIFTED: Readonly<Record<string, WindowCommand>> = {
  KeyN: { kind: "newFolder" },
  KeyV: { kind: "validate" },
};

/**
 * The command of a shortcut (SPEC-UI §4.9, §5.5), or `null`. ⌘ on macOS
 * (Ctrl elsewhere, the `CmdOrCtrl` of the menu); Ctrl on macOS stays with
 * the terminal (Ctrl-D, Ctrl-W).
 */
export function shortcutCommand(event: ShortcutKey, mac: boolean): WindowCommand | null {
  const command = mac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  if (!command || event.altKey) return null;
  const digit = /^Digit([1-9])$/.exec(event.code);
  if (digit && !event.shiftKey) return { kind: "showTab", index: Number(digit[1]) - 1 };
  if (event.code === "Backspace" && !event.shiftKey) {
    return editsText(event.target) ? null : { kind: "delete" };
  }
  return (event.shiftKey ? SHIFTED : PLAIN)[event.code] ?? null;
}
