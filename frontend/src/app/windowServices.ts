import type { QueryClient } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { createContext, useContext } from "react";

import { backgroundControl } from "../design/background/backgroundControl";
import { createSessionController } from "../features/sessions/controller";
import { createOutputBus } from "../features/sessions/outputBus";
import type { SessionServices } from "../features/sessions/sessionServices";
import { useSessionsStore } from "../features/sessions/store";
import type { SessionTransport } from "../ipc/sessions";
import { useWorkspace } from "../state/workspace";
import { type Connections, createConnections } from "./connections";
import { type CanReadyVault, vaultReady } from "./failures";

/** The services the window shares: the sessions and the connections. */
export interface WindowServices {
  sessions: SessionServices;
  connections: Connections;
}

/**
 * The services over `transport` (SPEC-UI §5.1). The host side of the contract
 * of `SessionServices`: links open in the browser with the opener plugin
 * (`http(s)` only, by the capability), a key in a terminal calms the
 * background, "Riprova" asks the login anew and keeps the new tab in front.
 */
export function createWindowServices(
  transport: SessionTransport,
  queryClient: QueryClient,
  features: { hasLoginDialog: () => boolean; canReadyVault: CanReadyVault },
): WindowServices {
  const output = createOutputBus();
  const controller = createSessionController({
    transport,
    store: useSessionsStore.getState,
    output,
  });
  const connections = createConnections({ queryClient, controller, ...features });
  return {
    connections,
    sessions: {
      controller,
      output,
      openLink: (url) => void openUrl(url).catch(() => undefined),
      onActivity: () => backgroundControl.noteSessionActivity(),
      // "Riprova" of a session: the vault first, if locked and it can be
      // readied (DEC-041.2), then the login of "ask at launch".
      onRetry: async (session) => {
        if (!(await vaultReady(queryClient, features.canReadyVault))) return null;
        return connections.askLogin(session.nodeId);
      },
      onReplaced: (previous, next) => useWorkspace.getState().tabReplaced(previous, next),
    },
  };
}

/** The services of the window; `App` provides them. */
export const WindowServicesContext = createContext<WindowServices | null>(null);

/** The services of the window. */
export function useWindowServices(): WindowServices {
  const services = useContext(WindowServicesContext);
  if (services === null) {
    // A mistake of the code that mounts the window, never shown to the user.
    throw new Error("the main window needs a WindowServicesContext");
  }
  return services;
}
