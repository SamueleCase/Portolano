import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { events, type MenuCommand } from "../bindings";
import { queryKeys } from "./keys";

/** What the interface does with the events that need more than a refresh. */
export interface BackendEventHandlers {
  /** Another program changed the document: offer "Ricarica". */
  onModifiedExternally?: (fileName: string) => void;
  /** The document file is gone: offer "Ricrea" or "Apri altro". */
  onMissing?: (fileName: string) => void;
  /** A menu item was chosen. */
  onMenu?: (command: MenuCommand) => void;
}

/**
 * Listens to the events of the backend (SPEC-UI §4.4) while mounted: a save
 * of the app and a locked vault make the cached data stale; the others go to
 * `handlers`.
 */
export function useBackendEvents(handlers: BackendEventHandlers = {}): void {
  const queryClient = useQueryClient();
  // The latest handlers, without subscribing again at every render.
  const current = useRef(handlers);
  useEffect(() => {
    current.current = handlers;
  });

  useEffect(() => {
    const subscriptions = [
      events.documentChanged.listen(
        () => void queryClient.invalidateQueries({ queryKey: queryKeys.document }),
      ),
      events.vaultLocked.listen(() => {
        void queryClient.invalidateQueries({ queryKey: queryKeys.vault });
        void queryClient.invalidateQueries({ queryKey: queryKeys.document });
      }),
      events.documentModifiedExternally.listen((event) =>
        current.current.onModifiedExternally?.(event.payload.fileName),
      ),
      events.documentMissing.listen((event) => {
        // The first-run screen and the list of recent documents show it.
        void queryClient.invalidateQueries({ queryKey: queryKeys.appStatus });
        void queryClient.invalidateQueries({ queryKey: queryKeys.recentDocuments });
        current.current.onMissing?.(event.payload.fileName);
      }),
      events.menuAction.listen((event) => current.current.onMenu?.(event.payload.command)),
    ].map((subscription) =>
      // A listener that could not be registered (no backend, as in a test
      // without IPC) leaves the interface without that event, not broken.
      subscription.catch(() => null),
    );
    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => unlisten?.());
      }
    };
  }, [queryClient]);
}
