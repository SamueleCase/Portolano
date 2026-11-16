import { afterEach, describe, expect, it } from "vitest";

import { commands } from "../bindings";
import { TEST_MASTER_PASSWORD, initialFakeState, installFakeBackend } from "./fakeBackend";
import { idOf, ok, unlocked } from "./fakeBackendFixtures";

/** The fake backend answers with the semantics of the Rust commands, errors included. */

describe("fake backend", () => {
  it("refuses a name a sibling already has, on the name field", async () => {
    installFakeBackend();

    const result = await commands.addNode({
      parent: null,
      name: "Clienti",
      kind: { type: "folder", credential: { mode: "inherit" } },
      details: {},
      secrets: {},
    });

    expect(result.status === "error" && result.error.code).toBe("invalid");
    expect(result.status === "error" && result.error.details).toEqual({
      kind: "invalid",
      field: "name",
    });
  });

  it("refuses to delete a credential other nodes use, listing them", async () => {
    installFakeBackend();

    const result = await commands.deleteNode(await idOf("Admin ACME"), {
      recursive: false,
      force: false,
    });

    expect(result.status === "error" && result.error.code).toBe("referenced");
  });

  it("puts a deleted node back with undo", async () => {
    installFakeBackend();
    const web01 = await idOf("web01");

    const deleted = await commands.deleteNode(web01, { recursive: false, force: false });
    if (deleted.status === "error") throw new Error(deleted.error.message);
    await commands.undoDelete(deleted.data.undoId);

    expect(await idOf("web01")).toBe(web01);
  });

  it("gives the users of a deleted credential their credential back with undo", async () => {
    unlocked();
    const acme = await idOf("ACME");

    const deleted = await commands.deleteNode(await idOf("Admin ACME"), {
      recursive: false,
      force: true,
    });
    if (deleted.status === "error") throw new Error(deleted.error.message);
    await commands.undoDelete(deleted.data.undoId);

    const details = await commands.nodeDetails(acme);
    expect(details.status === "ok" && details.data.credentialSpec?.mode).toBe("by_id");
  });

  it("asks for the vault when a secret is set while it is locked", async () => {
    installFakeBackend();

    const result = await commands.setSecret(await idOf("Admin ACME"), { kind: "password" }, "x");

    expect(result.status === "error" && result.error.code).toBe("vaultLocked");
  });

  it("refuses a wrong master password", async () => {
    installFakeBackend();

    const result = await commands.unlockVault("password-sbagliata");

    expect(result.status === "error" && result.error.code).toBe("vault");
  });

  it("unlocks with the test master password", async () => {
    installFakeBackend();

    const result = await commands.unlockVault(TEST_MASTER_PASSWORD);

    expect(result.status === "ok" && result.data.locked).toBe(false);
  });

  it("names a copy after its original", async () => {
    unlocked();

    const copy = await commands.duplicateNode(await idOf("web01"));

    expect(copy.status === "ok" && copy.data.path).toBe("/Clienti/ACME/web01 (copia)");
  });

  it("gives the tree a locked badge while the vault is locked", async () => {
    installFakeBackend();

    const tree = await commands.tree();

    expect(JSON.stringify(tree)).toContain('"secrets":"vaultLocked"');
  });

  it("gives a single note instead of the missing secrets while the vault is locked", async () => {
    installFakeBackend();

    const issues = await commands.validate();

    const codes = issues.status === "ok" ? issues.data.map((issue) => issue.code) : [];
    expect(codes).toContain("vault_locked");
    expect(codes).not.toContain("missing_secret");
  });

  it("refuses a move into a descendant", async () => {
    installFakeBackend();

    const result = await commands.moveNode(await idOf("Clienti"), await idOf("ACME"), {
      kind: "end",
    });

    expect(result.status === "error" && result.error.code).toBe("invalid");
  });

  it("answers a document command without a document with noDocument", async () => {
    const state = initialFakeState();
    state.document = null;
    installFakeBackend(state);

    const result = await commands.tree();

    expect(result.status === "error" && result.error.code).toBe("noDocument");
  });

  it("imports with the answers in the order of the requests", async () => {
    const state = unlocked();
    state.chosen.importFile = "/Users/utente-di-prova/infra.rtsz";
    const options = { into: null, onConflict: "fail" as const, includeTrash: false };
    const preview = ok(await commands.importPreview(options));

    const result = ok(
      await commands.importApply(
        options,
        preview.requests.map((request) => ({ ...request, value: "sentinel-import" })),
      ),
    );

    expect(result.secrets_written).toBe(1);
  });

  it("keeps no typed import password", async () => {
    const state = unlocked();
    state.chosen.importFile = "/Users/utente-di-prova/infra.rtsz";
    const options = { into: null, onConflict: "fail" as const, includeTrash: false };
    const preview = ok(await commands.importPreview(options));

    await commands.importApply(
      options,
      preview.requests.map((request) => ({ ...request, value: "sentinel-import" })),
    );

    expect(JSON.stringify(state)).not.toContain("sentinel-import");
  });
});

describe("fake backend for the main window", () => {
  it("writes the open document back where its missing file was", async () => {
    const state = initialFakeState();
    const path = state.document?.path ?? "";
    delete state.files[path];
    installFakeBackend(state);

    ok(await commands.recreateDocument());

    expect(state.files[path]).toBe(state.document);
  });

  it("gives the hosts of the jump hosts of a connection", async () => {
    installFakeBackend();

    const detail = ok(await commands.nodeDetails(await idOf("web01")));

    expect(detail.connection?.jumps).toEqual(["bastion.example.com"]);
  });
});

describe("fake backend seeded by an end-to-end test", () => {
  afterEach(() => {
    delete window.__portolanoFakeSeed;
  });

  it("starts without a document and without recent ones for the first run", async () => {
    window.__portolanoFakeSeed = { document: "none" };
    installFakeBackend();

    const status = ok(await commands.appStatus());

    expect(status.document).toBeNull();
    expect(status.lastDocument).toBeNull();
  });

  it("answers the native dialogs with the paths of the seed", async () => {
    window.__portolanoFakeSeed = {
      document: "none",
      dialogs: { newDocument: "/Users/utente-di-prova/infra/nuovo.json" },
    };
    installFakeBackend();

    expect(ok(await commands.chooseNewDocument())).toEqual({ fileName: "nuovo.json" });
  });

  it("starts with the vault unlocked when the seed asks", async () => {
    window.__portolanoFakeSeed = { vaultLocked: false };
    installFakeBackend();

    expect(ok(await commands.vaultStatus()).locked).toBe(false);
  });

  it("adds a connection whose credential does not resolve when the seed asks", async () => {
    window.__portolanoFakeSeed = { unresolvedCredential: true };
    installFakeBackend();

    const detail = ok(await commands.nodeDetails(await idOf("web03")));

    expect(detail.effective?.kind).toBe("unresolved");
  });

  it("leaves unanswered the commands the seed names", async () => {
    window.__portolanoFakeSeed = { unanswered: ["tree"] };
    installFakeBackend();

    const answer = await Promise.race([
      commands.tree().then(() => "answered"),
      new Promise((resolve) => setTimeout(() => resolve("waiting"), 20)),
    ]);

    expect(answer).toBe("waiting");
  });

  it("starts from the sample document without a seed", async () => {
    installFakeBackend();

    expect(ok(await commands.appStatus()).document?.name).toBe("Infrastruttura");
  });
});
