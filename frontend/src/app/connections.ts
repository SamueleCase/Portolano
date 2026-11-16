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
import { type CanReadyVault, reportFailure, runReported, vaultReady } from "./failures";

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
  /** Whether the dialog that readies the vault is mounted (unlock or creation). */
  canReadyVault: CanReadyVault;
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
  canReadyVault,
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

  /** Opens `nodeId` in a new tab, or in the place of `session` ("Riprova"). */
  const start = async (nodeId: string, session: Session | undefined) => {
    // The vault first, then the login: a command repeated after the unlock
    // does not ask the login again.
    if (!(await vaultReady(queryClient, canReadyVault))) return;
    const again = await askLogin(nodeId);
    if (again === null) return;
    const login = again.login;
    await runReported(
      async () => {
        if (session === undefined) {
          const id = await controller.open({
            nodeId,
            size: INITIAL_TERMINAL_SIZE,
            ...(login !== undefined && { login }),
          });
          remember(id, nodeId);
          useWorkspace.getState().showTab(id);
          return;
        }
        await controller.retry(session.id, {
          size: INITIAL_TERMINAL_SIZE,
          login,
          onReplaced: (next) => {
            const workspace = useWorkspace.getState();
            workspace.tabReplaced(session.id, next);
            workspace.showTab(next);
          },
        });
      },
      queryClient,
      canReadyVault,
    );
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
        await start(nodeId, existing);
      } catch (failure) {
        // The vault or the login could not be asked (the backend did not answer).
        reportFailure(failure, queryClient);
      } finally {
        opening.delete(nodeId);
      }
    },
  };
}
