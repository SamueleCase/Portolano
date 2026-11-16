import { afterEach, describe, expect, it } from "vitest";

import type { LoginRequest } from "../bindings";
import { answerLogin, openRequest, requestLogin, useRequests } from "./requests";

const KIOSK: LoginRequest = {
  node: "00000000-0000-4000-8000-000000000107",
  path: "/Clienti/ACME/kiosk",
  protocol: "rdp",
  ask_domain: true,
};

describe("the requests of the hook points", () => {
  afterEach(() => useRequests.getState().close());

  it("opens one request at a time, the last one", () => {
    openRequest({ kind: "import" });

    openRequest({ kind: "preferences" });

    expect(useRequests.getState().active).toEqual({ kind: "preferences" });
  });

  it("gives the login typed to the connection that asked for it", async () => {
    const asked = requestLogin(KIOSK);
    expect(useRequests.getState().active).toEqual({ kind: "login", request: KIOSK });

    answerLogin({ username: "operatore", password: "sentinel-login" });

    await expect(asked).resolves.toEqual({ username: "operatore", password: "sentinel-login" });
    expect(useRequests.getState().active).toBeNull();
  });

  it("never keeps the login in the store", async () => {
    const asked = requestLogin(KIOSK);

    answerLogin({ username: "operatore", password: "sentinel-login" });
    await asked;

    expect(JSON.stringify(useRequests.getState())).not.toContain("sentinel-login");
  });

  it("gives up the connection when the login dialog closes", async () => {
    const asked = requestLogin(KIOSK);

    useRequests.getState().close();

    await expect(asked).resolves.toBeNull();
  });

  it("gives up the connection when another request replaces the login dialog", async () => {
    const asked = requestLogin(KIOSK);

    openRequest({ kind: "preferences" });

    await expect(asked).resolves.toBeNull();
  });
});
