import { describe, expect, it } from "vitest";

import { commands } from "../../bindings";
import { installFakeBackend } from "../../test/fakeBackend";
import { idOf, ok } from "../../test/fakeBackendFixtures";
import { credentialOrigin } from "./credentialOrigin";

/** The origin of the credential of node `name` of the sample document. */
async function originOf(name: string) {
  installFakeBackend();
  return credentialOrigin(ok(await commands.nodeDetails(await idOf(name))));
}

describe("the origin of the effective credential", () => {
  it("names the folder it is inherited from, with the chain to the credential", async () => {
    expect(await originOf("web01")).toEqual({
      mode: "Ereditata dalla cartella ACME",
      source: "credenziale Clienti / Admin ACME",
      chain: ["web01", "ACME", "Admin ACME"],
    });
  });

  it("calls a folder's own credential its default one", async () => {
    expect((await originOf("ACME"))?.mode).toBe("Predefinita della cartella");
  });

  it("says a credential found by name was looked up by name", async () => {
    expect((await originOf("db01"))?.mode).toBe("Per nome");
  });

  it("says a credential asked at launch is asked at every connection", async () => {
    expect(await originOf("kiosk")).toMatchObject({ mode: "Chiedi all'avvio", chain: null });
  });

  it("counts the users of a credential node", async () => {
    expect((await originOf("Admin ACME"))?.source).toBe("usata da 5 nodi");
  });

  it("has nothing to say for an unsupported node", async () => {
    expect(await originOf("vnc-legacy")).toBeNull();
  });
});
