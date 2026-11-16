import { afterEach, describe, expect, it } from "vitest";

import type { LoginRequest } from "../bindings";
import { initialFakeState, installFakeBackend } from "../test/fakeBackend";
import {
  answerRequest,
  closeAllRequests,
  closeRequest,
  openRequest,
  pushRequest,
  requestLogin,
  requestVault,
  topRequest,
  useRequests,
} from "./requests";

const KIOSK: LoginRequest = {
  node: "00000000-0000-4000-8000-000000000107",
  path: "/Clienti/ACME/kiosk",
  protocol: "rdp",
  ask_domain: true,
};

const kinds = () => useRequests.getState().stack.map(({ request }) => request.kind);

/** The id of the request on top, which a test then answers. */
function topId(): number {
  const top = topRequest();
  if (!top) throw new Error("nessuna richiesta aperta");
  return top.id;
}

/** Lets the promise of a request register its answer. */
const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("the requests of the hook points", () => {
  afterEach(() => closeAllRequests());

  it("opens a request of the user when nothing is open", () => {
    expect(openRequest({ kind: "import" })).toBe(true);

    expect(kinds()).toEqual(["import"]);
  });

  it("refuses a request of the user while a dialog is open, instead of replacing it", () => {
    openRequest({ kind: "import" });

    expect(openRequest({ kind: "preferences" })).toBe(false);
    expect(kinds()).toEqual(["import"]);
  });

  it("puts a request of the code on top of the one open, which stays", () => {
    openRequest({ kind: "editor", target: { mode: "edit", id: "web01" } });

    pushRequest({ kind: "preferences" });

    expect(kinds()).toEqual(["editor", "preferences"]);
  });

  it("closes the request it is asked to close, and only that one", () => {
    openRequest({ kind: "import" });
    pushRequest({ kind: "preferences" });

    closeRequest(topId());

    expect(kinds()).toEqual(["import"]);
  });

  it("gives the login typed to the connection that asked for it", async () => {
    const asked = requestLogin(KIOSK);
    expect(topRequest()?.request).toEqual({ kind: "login", request: KIOSK });

    answerRequest(topId(), { username: "operatore", password: "sentinel-login" });

    await expect(asked).resolves.toEqual({ username: "operatore", password: "sentinel-login" });
    expect(kinds()).toEqual([]);
  });

  it("asks the login on top of the editor that needs it", async () => {
    openRequest({ kind: "editor", target: { mode: "edit", id: "kiosk" } });

    const asked = requestLogin(KIOSK);
    answerRequest(topId(), null);

    await expect(asked).resolves.toBeNull();
    expect(kinds()).toEqual(["editor"]);
  });

  it("answers a request by its id even when another is on top", async () => {
    const asked = requestLogin(KIOSK);
    const login = topId();
    pushRequest({ kind: "preferences" });

    answerRequest(login, { username: "operatore" });

    await expect(asked).resolves.toEqual({ username: "operatore" });
    expect(kinds()).toEqual(["preferences"]);
  });

  it("gives up the connection when the login dialog closes", async () => {
    const asked = requestLogin(KIOSK);

    closeRequest(topId());

    await expect(asked).resolves.toBeNull();
  });

  it("answers at once that an open vault is ready", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    installFakeBackend(state);

    await expect(requestVault()).resolves.toBe(true);
    expect(kinds()).toEqual([]);
  });

  it("asks to unlock a locked vault and tells whether it was", async () => {
    installFakeBackend();
    const ready = requestVault();
    await settled();
    expect(kinds()).toEqual(["unlockVault"]);

    answerRequest(topId(), true);

    await expect(ready).resolves.toBe(true);
  });

  it("asks to create a vault whose file does not exist yet", async () => {
    const state = initialFakeState();
    state.vault.exists = false;
    installFakeBackend(state);

    const ready = requestVault();
    await settled();

    expect(kinds()).toEqual(["createFileVault"]);
    closeRequest(topId());
    await expect(ready).resolves.toBe(false);
  });
});
