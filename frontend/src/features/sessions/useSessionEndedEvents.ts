import { useEffect } from "react";

import { events } from "../../bindings";
import { useSessionsStore } from "./store";

/**
 * Applies the `session-ended` events while mounted. The state channel of a
 * session already brings its last state; the event is the record the backend
 * keeps of every end, so the store is right even if the message of the channel
 * did not arrive. Applying it twice changes nothing, and an event of a session
 * the store does not hold (a tab closed or replaced, or a session that ended
 * before `open_session` answered, whose channel brings the end once it is
 * added) is ignored by the store.
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
