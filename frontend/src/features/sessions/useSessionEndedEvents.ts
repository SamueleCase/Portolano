import { useEffect } from "react";

import { events } from "../../bindings";
import { useSessionsStore } from "./store";

/**
 * Applies the `session-ended` events while mounted: the last state of a
 * session also reaches the tab whose state channel is gone (the window was
 * reloaded). An event of a session the interface does not know yet, one that
 * ended before `open_session` answered, changes nothing: its channel brings
 * the same state once the session is known.
 */
export function useSessionEndedEvents(): void {
  useEffect(() => {
    const subscription = events.sessionEnded
      .listen(({ payload }) => useSessionsStore.getState().stateChanged(payload.id, payload.state))
      // Without a backend (a test without IPC) the tabs keep their channels.
      .catch(() => null);
    return () => {
      void subscription.then((unlisten) => unlisten?.());
    };
  }, []);
}
