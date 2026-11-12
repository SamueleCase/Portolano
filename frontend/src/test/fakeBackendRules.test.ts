import { emit, listen } from "@tauri-apps/api/event";
import { describe, expect, it, vi } from "vitest";

import { commands } from "../bindings";
import { initialFakeState, installFakeBackend } from "./fakeBackend";
import { idAt, idOf, ok, refused, unlocked } from "./fakeBackendFixtures";

/** The rules the fake shares with the Rust commands (DEC-031 F3–F7 and the field of each error). */

describe("fake vault", () => {
  it("never locks the keychain", async () => {
    const state = initialFakeState();
    state.vault.kind = "keychain";
    installFakeBackend(state);

    const tree = ok(await commands.tree());

    expect(JSON.stringify(tree)).not.toContain("vaultLocked");
    expect(ok(await commands.vaultStatus()).locked).toBe(false);
  });

  it("refuses to unlock the keychain, on the backend field", async () => {
    const state = initialFakeState();
    state.vault.kind = "keychain";
    installFakeBackend(state);

    const error = refused(await commands.unlockVault("x"));

    expect(error.details).toEqual({ kind: "invalid", field: "vault.backend" });
  });

  it("answers the secrets of a node without secrets with the vault locked", async () => {
    installFakeBackend();

    expect(ok(await commands.secretStatus(await idOf("web01")))).toEqual([]);
  });

  it("refuses a deletion that removes secrets while the vault is locked", async () => {
    installFakeBackend();

    const error = refused(
      await commands.deleteNode(await idOf("deploy"), { recursive: false, force: true }),
    );

    expect(error.code).toBe("vaultLocked");
  });

  it("refuses an edit that clears a secret while the vault is locked", async () => {
    installFakeBackend();

    const error = refused(
      await commands.editNode(await idOf("Admin ACME"), {
        secrets: { password: { action: "clear" } },
      }),
    );

    expect(error.code).toBe("vaultLocked");
  });

  it("shows the effective credential as from an empty vault while it is locked", async () => {
    installFakeBackend();

    const details = ok(await commands.nodeDetails(await idOf("dc01")));

    expect(details.vaultLocked).toBe(true);
    expect(details.effective?.kind === "resolved" && details.effective.credential.password).toBe(
      "absent",
    );
  });

  it("does not mark as locked a detail that does not touch the vault", async () => {
    installFakeBackend();

    expect(ok(await commands.nodeDetails(await idOf("kiosk"))).vaultLocked).toBe(false);
  });

  it("brings the default backend back when the backend is reset", async () => {
    installFakeBackend();

    const entries = ok(await commands.configReset("vault.backend"));

    expect(entries).toContainEqual({ key: "vault.backend", value: "keychain" });
    expect(ok(await commands.vaultStatus()).kind).toBe("keychain");
  });
});

describe("fake nodes", () => {
  it("keeps a node whose move is refused", async () => {
    installFakeBackend();
    const web01 = await idOf("web01");

    refused(await commands.moveNode(web01, null, { kind: "before", id: web01 }));

    expect(await idOf("web01")).toBe(web01);
  });

  it("counts a jump host as a reference, besides the credential", async () => {
    unlocked();

    const preview = ok(await commands.previewDelete(await idOf("bastion")));

    expect(preview.references.map((reference) => reference.from_path)).toEqual([
      "/Clienti/ACME/web01",
    ]);
  });

  it("detaches only the credentials of the deleted subtree", async () => {
    unlocked();
    const web01 = await idOf("web01");

    const deleted = ok(
      await commands.deleteNode(await idOf("Rete"), { recursive: true, force: true }),
    );

    expect(deleted.outcome.detached).not.toContain(web01);
    expect(deleted.outcome.remaining_references.map((reference) => reference.from)).toContain(
      web01,
    );
  });

  it("names the field of a private key given to a connection", async () => {
    unlocked();

    const error = refused(
      await commands.editNode(await idOf("web01"), {
        secrets: { privateKey: { action: "set", value: "x" } },
      }),
    );

    expect(error.details).toEqual({ kind: "invalid", field: "private_key" });
  });

  it("names the port field of a port given to a folder", async () => {
    installFakeBackend();

    const error = refused(await commands.editNode(await idOf("Clienti"), { defaultPort: true }));

    expect(error.details).toEqual({ kind: "invalid", field: "port" });
  });

  it("makes the credential inline when a user is given to a connection", async () => {
    installFakeBackend();
    const web01 = await idOf("web01");

    ok(await commands.editNode(web01, { username: "ops" }));

    expect(ok(await commands.nodeDetails(web01)).credentialSpec).toEqual({
      mode: "inline",
      username: "ops",
      domain: null,
    });
  });

  it("refuses a key file that is not an absolute path", async () => {
    installFakeBackend();

    const error = refused(await commands.editNode(await idOf("deploy"), { keyFile: "id_ed25519" }));

    expect(error.details).toEqual({ kind: "invalid", field: "key_file" });
  });

  it("refuses an image that is not stored, on the icon field", async () => {
    installFakeBackend();

    const error = refused(
      await commands.editNode(await idOf("web01"), {
        details: { icon: { kind: "custom", sha256: "ab".repeat(32), mime: "image/png" } },
      }),
    );

    expect(error.details).toEqual({ kind: "invalid", field: "icon" });
  });

  it("renames a copied credential and points the copy to it", async () => {
    unlocked();

    const copy = ok(await commands.duplicateNode(await idOf("Clienti")));

    expect(copy.renamed.map((node) => node.path)).toEqual(["/Clienti (copia)/Admin ACME (copia)"]);
    const copiedAdmin = copy.renamed[0]?.id;
    const db01 = ok(await commands.nodeDetails(await idAt(`${copy.path}/ACME/db01`)));
    expect(db01.credentialSpec).toEqual({ mode: "by_id", target: copiedAdmin });
    const ts01 = ok(await commands.nodeDetails(await idAt(`${copy.path}/ts01`)));
    expect(ts01.connection?.settings["gateway"]).toEqual({
      host: "gw.example.com",
      credential: { mode: "by_id", $ref: copiedAdmin },
    });
  });

  it("leaves the original as it was when copying", async () => {
    unlocked();
    const admin = await idOf("Admin ACME");

    ok(await commands.duplicateNode(await idOf("Clienti")));

    const db01 = ok(await commands.nodeDetails(await idOf("db01")));
    expect(db01.effective).toMatchObject({
      kind: "resolved",
      credential: { origin: { node: admin } },
    });
    expect(ok(await commands.validate()).map((issue) => issue.code)).not.toContain(
      "by_name_ambiguous",
    );
  });

  it("counts a credential of the gateway in the settings as a reference", async () => {
    unlocked();

    const preview = ok(await commands.previewDelete(await idOf("Admin ACME")));

    expect(preview.references.map((reference) => reference.from_path).sort()).toEqual([
      "/Clienti/ACME",
      "/Clienti/ACME/db01",
      "/Clienti/ts01",
    ]);
  });

  it("lists the connections that inherit a credential to delete", async () => {
    unlocked();

    const preview = ok(await commands.previewDelete(await idOf("Admin ACME")));

    expect(preview.inherited.map((node) => node.path)).toEqual([
      "/Clienti/ACME/web01",
      "/Clienti/ACME/dc01",
    ]);
  });

  it("gives the gateway its credential back with undo", async () => {
    unlocked();
    const deleted = ok(
      await commands.deleteNode(await idOf("Admin ACME"), { recursive: false, force: true }),
    );
    const ts01 = await idOf("ts01");
    const detached = ok(await commands.nodeDetails(ts01)).connection?.settings["gateway"];
    expect(detached).toMatchObject({ credential: { mode: "prompt" } });

    const restored = ok(await commands.undoDelete(deleted.undoId));

    expect(restored.kept).toEqual([]);
    expect(ok(await commands.nodeDetails(ts01)).connection?.settings["gateway"]).toMatchObject({
      credential: { mode: "by_id" },
    });
  });

  it("lists as kept a node changed between deletion and undo", async () => {
    unlocked();
    const deleted = ok(
      await commands.deleteNode(await idOf("Admin ACME"), { recursive: false, force: true }),
    );
    const acme = await idOf("ACME");
    ok(await commands.editNode(acme, { credential: { mode: "none" } }));

    const restored = ok(await commands.undoDelete(deleted.undoId));

    expect(restored.kept.map((node) => node.id)).toEqual([acme]);
  });

  it("does not list as kept a node deleted between deletion and undo", async () => {
    unlocked();
    const deleted = ok(
      await commands.deleteNode(await idOf("Admin ACME"), { recursive: false, force: true }),
    );
    ok(await commands.deleteNode(await idOf("db01"), { recursive: false, force: false }));

    const restored = ok(await commands.undoDelete(deleted.undoId));

    expect(restored.kept).toEqual([]);
  });

  it("refuses an empty name when adding, on the name field", async () => {
    installFakeBackend();

    const error = refused(
      await commands.addNode({
        parent: null,
        name: " ",
        kind: { type: "folder", credential: { mode: "inherit" } },
      }),
    );

    expect(error.details).toEqual({ kind: "invalid", field: "name" });
  });

  it("refuses an empty name when editing, on the name field", async () => {
    installFakeBackend();

    const error = refused(await commands.editNode(await idOf("web01"), { name: "" }));

    expect(error.details).toEqual({ kind: "invalid", field: "name" });
  });

  it("refuses a password on an unsupported node, on the password field", async () => {
    unlocked();

    const error = refused(
      await commands.editNode(await idOf("vnc-legacy"), {
        secrets: { password: { action: "set", value: "x" } },
      }),
    );

    expect(error.details).toEqual({ kind: "invalid", field: "password" });
  });

  it("answers the key file chosen in the dialog", async () => {
    const state = installFakeBackend();
    state.dialogs.keyFile = "/Users/utente-di-prova/.ssh/id_ed25519";

    expect(ok(await commands.chooseKeyFile())).toBe("/Users/utente-di-prova/.ssh/id_ed25519");
  });
});

describe("fake events and expiry", () => {
  it("sends document-changed after a save", async () => {
    unlocked();
    const changed = vi.fn();
    const unlisten = await listen("document-changed", changed);

    ok(await commands.duplicateNode(await idOf("web01")));

    await vi.waitFor(() => expect(changed).toHaveBeenCalled());
    unlisten();
  });

  it("sends vault-locked after lock_vault", async () => {
    unlocked();
    const locked = vi.fn();
    const unlisten = await listen("vault-locked", locked);

    ok(await commands.lockVault());

    await vi.waitFor(() => expect(locked).toHaveBeenCalled());
    unlisten();
  });

  it("forgets a deletion once it can no longer be undone", async () => {
    const state = unlocked();
    let now = 0;
    state.clock = () => now;
    const deleted = ok(
      await commands.deleteNode(await idOf("web01"), { recursive: false, force: false }),
    );

    now = 10_000;
    const error = refused(await commands.undoDelete(deleted.undoId));

    expect(error.code).toBe("notFound");
  });

  it("lets a test send the events of the watcher itself", async () => {
    installFakeBackend();
    const missing = vi.fn();
    const unlisten = await listen("document-missing", missing);

    await emit("document-missing", { fileName: "portolano.json" });

    await vi.waitFor(() => expect(missing).toHaveBeenCalled());
    unlisten();
  });
});
