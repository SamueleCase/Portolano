import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";

import type { TerminalIo } from "../../components/terminal/terminalIo";
import { presentFailure } from "../../ipc/errors";
import type { SessionId, SessionLogin } from "../../ipc/sessions";
import type { SessionController } from "./controller";
import type { OutputBus } from "./outputBus";
import type { Session } from "./store";

/**
 * What the session views work with: the app gives them the transport of the
 * bindings, the gallery and the tests the fake one.
 */
export interface SessionServices {
  controller: SessionController;
  output: OutputBus;
  /** Opens an `http(s)` link of a terminal in the browser. */
  openLink: (url: string) => void;
  /** A key typed in a terminal: the host calms the background while the user works. */
  onActivity?: () => void;
  /**
   * Asked before "Riprova" opens `session` again (DEC-033.3): for a credential
   * "ask at launch" the host asks the login anew and returns it; `{}` retries
   * without a login, `null` gives up. The login goes to the backend and is not
   * kept. Without it, "Riprova" retries without a login.
   */
  onRetry?: (session: Session) => Promise<{ login?: SessionLogin } | null>;
  /** "Riprova" put session `next` in the place of `previous`: the host keeps it selected. */
  onReplaced?: (previous: SessionId, next: SessionId) => void;
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

/** The actions of the header of a session tab. */
export type SessionAction = "retry" | "disconnect" | "close";

/**
 * The actions of a session tab, one at a time: while one runs, `pending`
 * names it and another click does nothing; a failure leaves its message in
 * `error`.
 */
export function useSessionAction(): {
  run: (name: SessionAction, action: () => Promise<unknown>) => void;
  pending: SessionAction | undefined;
  error: string | undefined;
} {
  const [error, setError] = useState<string>();
  const [pending, setPending] = useState<SessionAction>();
  // Read synchronously: two clicks before a render must not both start.
  const running = useRef(false);
  return {
    run: (name, action) => {
      if (running.current) return;
      running.current = true;
      setPending(name);
      setError(undefined);
      action()
        .catch((failure: unknown) => setError(presentFailure(failure).message))
        .finally(() => {
          running.current = false;
          setPending(undefined);
        });
    },
    pending,
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
