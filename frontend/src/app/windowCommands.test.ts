import { beforeEach, describe, expect, it } from "vitest";

import { DETAIL_TAB, useWorkspace } from "../state/workspace";
import type { WindowActions } from "./windowActions";
import {
  type CommandContext,
  runWindowCommand,
  shortcutCommand,
  type ShortcutKey,
} from "./windowCommands";

/** Actions that record their calls; the editor is mounted unless `offered` says not. */
function recordingActions(offered = true) {
  const calls: string[] = [];
  const record =
    (name: string) =>
    (...args: unknown[]) =>
      calls.push([name, ...args.map(String)].join(" "));
  // Every action records its name: the proxy stands for the whole interface.
  const actions = new Proxy({} as WindowActions, {
    get: (_target, name: string) => (name === "offers" ? () => offered : record(name)),
  });
  return { actions, calls };
}

const nothing: CommandContext = { selection: undefined, tabs: [DETAIL_TAB], modalOpen: false };
const onSsh: CommandContext = { ...nothing, selection: { id: "web01", kind: "ssh" } };

function key(code: string, modifiers: Partial<ShortcutKey> = {}): ShortcutKey {
  return {
    code,
    metaKey: true,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    target: null,
    ...modifiers,
  };
}

describe("the commands of the menu and the shortcuts", () => {
  beforeEach(() => useWorkspace.getState().documentSwitched());

  it("connects to the selected connection", () => {
    const { actions, calls } = recordingActions();

    runWindowCommand({ kind: "connect" }, actions, onSsh);

    expect(calls).toEqual(["connect web01"]);
  });

  it("does not connect to a node that is not a connection", () => {
    const { actions, calls } = recordingActions();

    runWindowCommand({ kind: "connect" }, actions, {
      ...nothing,
      selection: { id: "deploy", kind: "credential" },
    });

    expect(calls).toEqual([]);
  });

  it("does not edit an unsupported node, which is read-only", () => {
    const { actions, calls } = recordingActions();

    runWindowCommand({ kind: "edit" }, actions, {
      ...nothing,
      selection: { id: "vnc", kind: "unsupported" },
    });

    expect(calls).toEqual([]);
  });

  it("does nothing behind an open dialog", () => {
    const { actions, calls } = recordingActions();

    runWindowCommand({ kind: "delete" }, actions, { ...onSsh, modalOpen: true });

    expect(calls).toEqual([]);
  });

  it("opens the menu of «+ Connessione» to choose the protocol", () => {
    const { actions } = recordingActions();

    runWindowCommand({ kind: "newConnection" }, actions, nothing);

    expect(useWorkspace.getState().connectionMenuOpen).toBe(true);
  });

  it("never closes the tab of the detail", () => {
    const { actions, calls } = recordingActions();

    runWindowCommand({ kind: "closeTab" }, actions, nothing);

    expect(calls).toEqual([]);
  });

  it("brings in front the tab of its number", () => {
    const { actions } = recordingActions();

    runWindowCommand({ kind: "showTab", index: 1 }, actions, {
      ...nothing,
      tabs: [DETAIL_TAB, "fake-session-1"],
    });

    expect(useWorkspace.getState().activeTab).toBe("fake-session-1");
  });

  it("reads the shortcuts of the menu with ⌘ on macOS", () => {
    expect(shortcutCommand(key("KeyE"), true)).toEqual({ kind: "edit" });
    expect(shortcutCommand(key("KeyN", { shiftKey: true }), true)).toEqual({ kind: "newFolder" });
    expect(shortcutCommand(key("KeyV", { shiftKey: true }), true)).toEqual({ kind: "validate" });
    expect(shortcutCommand(key("Digit3"), true)).toEqual({ kind: "showTab", index: 2 });
  });

  it("leaves Ctrl to the terminal on macOS", () => {
    expect(shortcutCommand(key("KeyD", { metaKey: false, ctrlKey: true }), true)).toBeNull();
  });

  it("leaves ⌘⌫ to a text field, where it deletes text", () => {
    const field = document.createElement("input");

    expect(shortcutCommand(key("Backspace", { target: field }), true)).toBeNull();
    expect(shortcutCommand(key("Backspace"), true)).toEqual({ kind: "delete" });
  });

  it("does not read keys without ⌘", () => {
    expect(shortcutCommand(key("KeyE", { metaKey: false }), true)).toBeNull();
  });
});
