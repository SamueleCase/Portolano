import { useQueryClient } from "@tanstack/react-query";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";

import type { LoginInput_Deserialize, LoginRequest } from "../../bindings";
import { loginRequestQuery } from "../../ipc/queries";
import { LoginDialog } from "./LoginDialog";

/**
 * What a connection needs before opening, with the shape of `onRetry` of
 * the sessions (DEC-033.8): `{ login }` typed by the user, `{}` when the
 * credential is not "Chiedi all'avvio", `null` when the user gives up.
 */
export type LoginAnswer = { login?: LoginInput_Deserialize } | null;

/** What the caller of {@link useLoginPrompt} mounts and calls. */
export interface LoginPrompt {
  /**
   * Asks `login_request` for connection `nodeId` and, if its credential is
   * "Chiedi all'avvio", the login with the dialog. The login goes to the
   * caller, which passes it to the backend; the hook keeps nothing of it.
   * Rejects with the failure of `login_request`.
   */
  askLogin: (nodeId: string) => Promise<LoginAnswer>;
  /** The dialog to render while it asks. */
  dialog: ReactNode;
}

/**
 * The credentials at launch (DEC-016.3, DEC-035.3): used by the connection
 * of the main window and by "Riprova" of the sessions.
 */
export function useLoginPrompt(): LoginPrompt {
  const queryClient = useQueryClient();
  const [request, setRequest] = useState<LoginRequest | null>(null);
  const resolve = useRef<((answer: LoginAnswer) => void) | undefined>(undefined);

  const finish = (answer: LoginAnswer) => {
    setRequest(null);
    resolve.current?.(answer);
    resolve.current = undefined;
  };

  // Unmounted while asking: the caller gives up, as with "Annulla".
  useEffect(() => () => resolve.current?.(null), []);

  const askLogin = useCallback(
    async (nodeId: string): Promise<LoginAnswer> => {
      const asked = await queryClient.fetchQuery({ ...loginRequestQuery(nodeId), staleTime: 0 });
      if (asked === null) return {};
      // A request still open gives up for the new one.
      resolve.current?.(null);
      return new Promise<LoginAnswer>((done) => {
        resolve.current = done;
        setRequest(asked);
      });
    },
    [queryClient],
  );

  const dialog = (
    <LoginDialog
      request={request}
      onLogin={(login) => finish({ login })}
      onClose={() => finish(null)}
    />
  );
  return { askLogin, dialog };
}
