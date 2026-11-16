import type { QueryClient } from "@tanstack/react-query";

import type { SessionController } from "../features/sessions/controller";
import { isOver } from "../features/sessions/sessionState";
import { type Session, useSessionsStore } from "../features/sessions/store";
import { indexTree } from "../features/tree/treeModel";
import { texts } from "../i18n/it";
import { loginRequestQuery, treeQuery } from "../ipc/queries";
import type { SessionLogin } from "../ipc/sessions";
import { requestLogin } from "../state/requests";
import { showToast } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { runReported } from "./failures";

/** The size a terminal starts with; it fits its tab as soon as it shows. */
export const INITIAL_TERMINAL_SIZE = { cols: 80, rows: 24 };

/** How a session opens: with a login asked at launch, or without. */
export type LoginAnswer = { login?: SessionLogin } | null;

/** What the connections of the window work with. */
export interface ConnectionsDeps {
  queryClient: QueryClient;
  controller: SessionController;
  /** Whether the login dialog is mounted (hook point `login`). */
  hasLoginDialog: () => boolean;
  /** Whether the unlock dialog is mounted (hook point `unlockVault`). */
  hasUnlockDialog: () => boolean;
}

/** Opening sessions from the tree, the toolbar, the detail and the menus. */
export interface Connections {
  /**
   * Connects to `nodeId` (inventario §5.1): a session of the node still
   * running comes in front, one that is over opens again in its tab
   * ("Riprova"), otherwise a new tab opens. A locked vault is unlocked first
   * if it can be (DEC-041.2).
   */
  connect: (nodeId: string) => Promise<void>;
  /**
   * The login of `nodeId`, if its credential is "ask at launch": `{}` when
   * none is needed, `null` when the user gives up. Never kept.
   */
  askLogin: (nodeId: string) => Promise<LoginAnswer>;
}

/** The connections over `deps`. */
export function createConnections({
  queryClient,
  controller,
  hasLoginDialog,
  hasUnlockDialog,
}: ConnectionsDeps): Connections {
  // A double click or a repeated shortcut must not open two sessions.
  const opening = new Set<string>();

  /** The summary of a connection, the title of its tab if the node goes away. */
  const remember = (session: string, nodeId: string) => {
    const view = queryClient.getQueryData(treeQuery.queryKey);
    const summary = view ? (indexTree(view).entries.get(nodeId)?.node.summary ?? "") : "";
    useWorkspace.getState().sessionOpened(session, summary);
  };

  const askLogin = async (nodeId: string): Promise<LoginAnswer> => {
    // Always asked anew: the credential may have changed since.
    const request = await queryClient.fetchQuery({ ...loginRequestQuery(nodeId), staleTime: 0 });
    if (request === null) return {};
    if (!hasLoginDialog()) {
      showToast({ kind: "error", text: texts.app.connection.noLoginDialog });
      return null;
    }
    const login = await requestLogin(request);
    return login === null ? null : { login };
  };

  const reopen = async (session: Session) => {
    const again = await askLogin(session.nodeId);
    if (again === null) return;
    await controller.retry(session.id, {
      size: INITIAL_TERMINAL_SIZE,
      login: again.login,
      onReplaced: (next) => {
        const workspace = useWorkspace.getState();
        workspace.tabReplaced(session.id, next);
        workspace.showTab(next);
      },
    });
  };

  const open = async (nodeId: string) => {
    const again = await askLogin(nodeId);
    if (again === null) return;
    const id = await controller.open({
      nodeId,
      size: INITIAL_TERMINAL_SIZE,
      ...(again.login !== undefined && { login: again.login }),
    });
    remember(id, nodeId);
    useWorkspace.getState().showTab(id);
  };

  return {
    askLogin,
    async connect(nodeId) {
      if (opening.has(nodeId)) return;
      const existing = useSessionsStore
        .getState()
        .sessions.findLast((session) => session.nodeId === nodeId);
      if (existing !== undefined && !isOver(existing.state)) {
        useWorkspace.getState().showTab(existing.id);
        return;
      }
      opening.add(nodeId);
      try {
        await runReported(
          () => (existing === undefined ? open(nodeId) : reopen(existing)),
          queryClient,
          hasUnlockDialog,
        );
      } finally {
        opening.delete(nodeId);
      }
    },
  };
}
