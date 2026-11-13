import { createContext, useContext, useEffect, useMemo, useState } from "react";

import type { TerminalIo } from "../../components/terminal/terminalIo";
import { presentFailure } from "../../ipc/errors";
import type { SessionId } from "../../ipc/sessions";
import type { SessionController } from "./controller";
import type { OutputBus } from "./outputBus";

/**
 * What the session views work with: the app gives them the transport of the
 * bindings, the gallery and the tests the fake one.
 */
export interface SessionServices {
  controller: SessionController;
  output: OutputBus;
  /** Opens an `http(s)` link of a terminal in the browser. */
  openLink: (url: string) => void;
}

/** The services of the session views; the window that mounts them provides it. */
export const SessionServicesContext = createContext<SessionServices | null>(null);

/** The services of the session views. */
export function useSessionServices(): SessionServices {
  const services = useContext(SessionServicesContext);
  if (services === null) {
    // A mistake of the code that mounts the views, never shown to the user.
    throw new Error("session views need a SessionServicesContext");
  }
  return services;
}

/**
 * The terminal of session `id`: the output kept by the bus, then each new
 * one; what is typed and the new sizes go to its program.
 */
export function useTerminalIo(id: SessionId): TerminalIo {
  const { controller, output } = useSessionServices();
  return useMemo(
    () => ({
      subscribe: (onOutput) => output.subscribe(id, onOutput),
      // A failure (the session is gone) is shown by the state of the tab.
      input: (data) => void controller.input(id, data).catch(() => undefined),
      resize: (cols, rows) => void controller.resize(id, { cols, rows }).catch(() => undefined),
    }),
    [controller, output, id],
  );
}

/** An action of a session tab, with the message of its last failure. */
export function useSessionAction(): {
  run: (action: () => Promise<unknown>) => void;
  error: string | undefined;
} {
  const [error, setError] = useState<string>();
  return {
    run: (action) => {
      setError(undefined);
      action().catch((failure: unknown) => setError(presentFailure(failure).message));
    },
    error,
  };
}

/** The current time, every second while `ticking`: a duration that runs. */
export function useNow(ticking: boolean): number {
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    if (!ticking) return undefined;
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [ticking]);
  return now;
}
