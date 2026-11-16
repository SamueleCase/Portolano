import { create } from "zustand";

import type { LoginRequest, NodeRef, SecretSlotInput } from "../bindings";
import type { SessionLogin } from "../ipc/sessions";

/** The kinds of node "+ Connessione", "+ Cartella" and "+ Credenziale" create. */
export type NewNodeKind = "ssh" | "rdp" | "folder" | "credential";

/** What the editor opens on (ambiguity 10: the parent comes from the selection). */
export type EditorTarget =
  { mode: "add"; kind: NewNodeKind; parent: string | null } | { mode: "edit"; id: string };

/** A request of the window to a function mounted at a hook point (`app/features.ts`). */
export type HostRequest =
  | { kind: "editor"; target: EditorTarget }
  | { kind: "password"; node: NodeRef; field: SecretSlotInput }
  | { kind: "unlockVault" }
  | { kind: "createFileVault" }
  | { kind: "login"; request: LoginRequest }
  | { kind: "import" }
  | { kind: "preferences" };

/** An open request, with the id that closes it. */
export interface OpenRequest {
  id: number;
  request: HostRequest;
}

/**
 * The open requests, a stack (DEC-041.1): the last one is on top. A request
 * of the user opens only with nothing open; a request made by the code of an
 * open function (the editor that needs the vault) goes on top of it, and the
 * one below stays mounted with its state.
 */
export interface RequestsState {
  stack: OpenRequest[];
}

export const useRequests = create<RequestsState>()(() => ({ stack: [] }));

let nextId = 0;

/**
 * Who waits for the answer of a request, by its id: a login, or whether the
 * vault was unlocked. The answers never enter the store (DEC-033.8, SPEC-UI
 * §1.2): the dialog hands them over, the caller gets them, nothing keeps them.
 */
const waiting = new Map<number, (answer: SessionLogin | boolean | null) => void>();

function push(request: HostRequest): number {
  nextId += 1;
  const id = nextId;
  useRequests.setState(({ stack }) => ({ stack: [...stack, { id, request }] }));
  return id;
}

/** Removes request `id`, giving up its answer if someone waits for it. */
function settle(id: number, answer: SessionLogin | boolean | null): void {
  const resolve = waiting.get(id);
  waiting.delete(id);
  useRequests.setState(({ stack }) => ({ stack: stack.filter((open) => open.id !== id) }));
  resolve?.(answer);
}

/** The request on top, if any. */
export function topRequest(): OpenRequest | undefined {
  return useRequests.getState().stack.at(-1);
}

/**
 * A request of the user (toolbar, menu, shortcut, detail): opens only when no
 * other is open, otherwise it is refused. Returns whether it opened.
 */
export function openRequest(request: HostRequest): boolean {
  if (useRequests.getState().stack.length > 0) return false;
  push(request);
  return true;
}

/** A request made by the code of an open function: on top of the others. */
export function pushRequest(request: HostRequest): void {
  push(request);
}

/** Request `id` is over (its dialog closed): one that waited gives up. */
export function closeRequest(id: number): void {
  settle(id, null);
}

/**
 * Asks the login of a credential "ask at launch" (DEC-016.3) with the dialog
 * of the hook point `login`, on top of what is open; resolves to it, or to
 * `null` if the user gives up. The login reaches the caller only.
 */
export function requestLogin(request: LoginRequest): Promise<SessionLogin | null> {
  const id = push({ kind: "login", request });
  return new Promise((resolve) => {
    waiting.set(id, (answer) => resolve(typeof answer === "object" ? answer : null));
  });
}

/** The login dialog on top is done: `login` goes to its connection, `null` cancels it. */
export function answerLogin(login: SessionLogin | null): void {
  const top = topRequest();
  if (top?.request.kind === "login") settle(top.id, login);
}

/**
 * Asks to unlock the file vault with the dialog of the hook point
 * `unlockVault`, on top of what is open (DEC-041.1); resolves to whether it
 * was unlocked.
 */
export function requestUnlock(): Promise<boolean> {
  const id = push({ kind: "unlockVault" });
  return new Promise((resolve) => {
    waiting.set(id, (answer) => resolve(answer === true));
  });
}

/**
 * The vault dialog on top (unlock or creation) is done: `done` tells whether
 * the vault is now open.
 */
export function answerVault(done: boolean): void {
  const top = topRequest();
  if (top?.request.kind === "unlockVault" || top?.request.kind === "createFileVault") {
    settle(top.id, done);
  }
}

/** Closes every request: the document changed under them. */
export function closeAllRequests(): void {
  for (const { id } of [...useRequests.getState().stack].reverse()) settle(id, null);
}
