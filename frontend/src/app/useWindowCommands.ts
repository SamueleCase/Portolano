import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef } from "react";

import type { WorkTab } from "../components/Tabs";
import { backgroundControl } from "../design/background/backgroundControl";
import { indexTree } from "../features/tree/treeModel";
import { treeQuery } from "../ipc/queries";
import { useRequests } from "../state/requests";
import { useWorkspace } from "../state/workspace";
import type { WindowActions } from "./windowActions";
import { runWindowCommand, shortcutCommand, type WindowCommand } from "./windowCommands";

/** Whether the shortcuts use ⌘ (macOS) or Ctrl, as `CmdOrCtrl` of the menu. */
export const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.userAgent);

/**
 * Whether a dialog or the editor is open: a request of the hook points, or
 * any `Dialog` / `SidePanel` of the window (the confirmation of a deletion,
 * "Sposta in…"), which all count themselves in the background control
 * through `useModal` (it slows the background behind them).
 */
export function modalOpen(): boolean {
  return useRequests.getState().stack.length > 0 || backgroundControl.inputs().openDialogs > 0;
}

/**
 * The function that performs a command of the menu or of a shortcut with the
 * current selection and tabs (read when it runs, not when it was made).
 */
export function useCommandRunner(
  actions: WindowActions,
  tabs: readonly WorkTab[],
): (command: WindowCommand) => void {
  const queryClient = useQueryClient();
  const currentTabs = useRef(tabs);
  useEffect(() => {
    currentTabs.current = tabs;
  });
  return useCallback(
    (command: WindowCommand) => {
      const view = queryClient.getQueryData(treeQuery.queryKey);
      const { selectedId } = useWorkspace.getState();
      const entry =
        view && selectedId !== null ? indexTree(view).entries.get(selectedId) : undefined;
      runWindowCommand(command, actions, {
        selection: entry && { id: entry.node.id, kind: entry.kind },
        tabs: currentTabs.current.map((tab) => tab.id),
        modalOpen: modalOpen(),
      });
    },
    [queryClient, actions],
  );
}

/**
 * The shortcuts of SPEC-UI §4.9 and §5.5 while mounted. A shortcut handled
 * here is not passed on (`preventDefault`): WKWebView gives the page the key
 * first, so the native menu does not perform the same command again.
 */
export function useShortcuts(run: (command: WindowCommand) => void): void {
  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      const command = shortcutCommand(event, isMac);
      if (command === null) return;
      event.preventDefault();
      run(command);
    };
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, [run]);
}
