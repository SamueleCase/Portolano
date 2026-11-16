import { describe, expect, it } from "vitest";

import { commands } from "../../bindings";
import { installFakeBackend } from "../../test/fakeBackend";
import { ok } from "../../test/fakeBackendFixtures";
import { rowLabel } from "./rowLabel";
import { indexTree } from "./treeModel";

async function entryOf(name: string) {
  installFakeBackend();
  const index = indexTree(ok(await commands.tree()));
  const entry = [...index.entries.values()].find(({ node }) => node.name === name);
  if (!entry) throw new Error(`nodo ${name} assente`);
  return entry;
}

describe("the accessible name of a row", () => {
  it("says the state of the session of the node", async () => {
    const bastion = await entryOf("bastion");

    expect(
      rowLabel(bastion, { kind: "host", host: "bastion.example.com" }, "connected", false),
    ).toBe("bastion, Connessione SSH, bastion.example.com, connesso");
  });

  it("says a node is a favorite", async () => {
    const web01 = await entryOf("web01");

    expect(rowLabel(web01, undefined, "idle", false)).toBe("web01, Connessione SSH, preferito");
  });

  it("counts the content of a closed folder only", async () => {
    const acme = await entryOf("ACME");
    const meta = { kind: "descendants", count: 4 } as const;

    expect(rowLabel(acme, meta, "idle", false)).toBe("ACME, Cartella, 4 elementi");
    expect(rowLabel(acme, meta, "idle", true)).toBe("ACME, Cartella");
  });
});
