import { create } from "zustand";

import type { LoginRequest, NodeRef, SecretSlotInput } from "../bindings";
import type { SessionLogin } from "../ipc/sessions";

/** The kinds of node "+ Connessione", "+ Cartella" and "+ Credenziale" create. */
export type NewNodeKind = "ssh" | "rdp" | "folder" | "credential";

/** What the editor opens on (ambiguity 10: the parent comes from the selection). */
export type EditorTarget =
  { mode: "add"; kind: NewNodeKind; parent: string | null } | { mode: "edit"; id: string };

/**
 * A request of the window to a function mounted at a hook point
 * (`app/features.ts`): one at a time, the last one wins.
 */
export type HostRequest =
  | { kind: "editor"; target: EditorTarget }
  | { kind: "password"; node: NodeRef; field: SecretSlotInput }
  | { kind: "unlockVault" }
  | { kind: "createFileVault" }
  | { kind: "login"; request: LoginRequest }
  | { kind: "import" }
  | { kind: "preferences" };

export interface RequestsState {
  /** The request being answered; none with nothing open. */
  active: HostRequest | null;
  /** Opens `request`, in place of the one open. */
  open: (request: HostRequest) => void;
  /** The function closed its dialog or panel. */
  close: () => void;
}

/**
 * Who waits for the login of the open `login` request. The login itself never
 * enters the store (DEC-033.8, SPEC-UI §1.2): the dialog hands it to
 * `answerLogin`, which gives it to the waiting connection and forgets it.
 */
let waitingLogin: ((login: SessionLogin | null) => void) | undefined;

function giveUpLogin(): void {
  const waiting = waitingLogin;
  waitingLogin = undefined;
  waiting?.(null);
}

export const useRequests = create<RequestsState>()((set) => ({
  active: null,
  open: (request) => {
    // A new request replaces a login dialog: its connection does not start.
    giveUpLogin();
    set({ active: request });
  },
  close: () => {
    giveUpLogin();
    set({ active: null });
  },
}));

/** Opens `request` from outside React (menus, shortcuts, commands). */
export function openRequest(request: HostRequest): void {
  useRequests.getState().open(request);
}

/**
 * Asks the login of a credential "ask at launch" (DEC-016.3) with the dialog
 * of the `login` hook point; resolves to it, or to `null` if the user gives
 * up. The login reaches the caller only, which passes it to the backend.
 */
export function requestLogin(request: LoginRequest): Promise<SessionLogin | null> {
  const { open } = useRequests.getState();
  open({ kind: "login", request });
  return new Promise((resolve) => {
    waitingLogin = resolve;
  });
}

/** The login dialog is done: `login` goes to the waiting connection, `null` cancels it. */
export function answerLogin(login: SessionLogin | null): void {
  const waiting = waitingLogin;
  waitingLogin = undefined;
  useRequests.setState({ active: null });
  waiting?.(login);
}
