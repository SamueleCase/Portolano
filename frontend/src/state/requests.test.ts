import { afterEach, describe, expect, it } from "vitest";

import type { LoginRequest } from "../bindings";
import {
  answerLogin,
  answerVault,
  closeAllRequests,
  closeRequest,
  openRequest,
  pushRequest,
  requestLogin,
  requestUnlock,
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
    const top = topRequest();

    if (top) closeRequest(top.id);

    expect(kinds()).toEqual(["import"]);
  });

  it("gives the login typed to the connection that asked for it", async () => {
    const asked = requestLogin(KIOSK);
    expect(topRequest()?.request).toEqual({ kind: "login", request: KIOSK });

    answerLogin({ username: "operatore", password: "sentinel-login" });

    await expect(asked).resolves.toEqual({ username: "operatore", password: "sentinel-login" });
    expect(kinds()).toEqual([]);
  });

  it("asks the login on top of the editor that needs it", async () => {
    openRequest({ kind: "editor", target: { mode: "edit", id: "kiosk" } });

    const asked = requestLogin(KIOSK);
    answerLogin(null);

    await expect(asked).resolves.toBeNull();
    expect(kinds()).toEqual(["editor"]);
  });

  it("does not answer a login that is not on top", async () => {
    const asked = requestLogin(KIOSK);
    pushRequest({ kind: "preferences" });

    answerLogin({ username: "operatore" });

    expect(kinds()).toEqual(["login", "preferences"]);
    closeAllRequests();
    await expect(asked).resolves.toBeNull();
  });

  it("gives up the connection when the login dialog closes", async () => {
    const asked = requestLogin(KIOSK);
    const top = topRequest();

    if (top) closeRequest(top.id);

    await expect(asked).resolves.toBeNull();
  });

  it("tells the caller whether the vault was unlocked", async () => {
    const unlocked = requestUnlock();

    answerVault(true);

    await expect(unlocked).resolves.toBe(true);
  });

  it("tells the caller the vault stays locked when the dialog closes", async () => {
    const unlocked = requestUnlock();
    const top = topRequest();

    if (top) closeRequest(top.id);

    await expect(unlocked).resolves.toBe(false);
  });
});
