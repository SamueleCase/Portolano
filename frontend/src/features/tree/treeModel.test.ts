import { describe, expect, it } from "vitest";

import { commands } from "../../bindings";
import { installFakeBackend } from "../../test/fakeBackend";
import { idOf, ok } from "../../test/fakeBackendFixtures";
import {
  ancestorsOf,
  iconChoice,
  indexTree,
  nodeKindOf,
  parentForNew,
  readablePath,
  siblingPlacement,
  type TreeIndex,
  visibleIds,
} from "./treeModel";

/** The tree of the sample document of the fake backend. */
async function sampleTree(): Promise<TreeIndex> {
  installFakeBackend();
  return indexTree(ok(await commands.tree()));
}

/** The names of `ids`, in tree order. */
function names(index: TreeIndex, ids: ReadonlySet<string> | null): string[] {
  return [...index.entries.values()]
    .filter(({ node }) => ids === null || ids.has(node.id))
    .map(({ node }) => node.name);
}

describe("the model of the tree", () => {
  it("draws a connection by its protocol", () => {
    expect(nodeKindOf({ kind: "connection", protocol: "rdp" })).toBe("rdp");
    expect(nodeKindOf({ kind: "connection", protocol: "ssh" })).toBe("ssh");
    expect(nodeKindOf({ kind: "folder", protocol: null })).toBe("folder");
  });

  it("takes the file name of a custom icon from its URL", () => {
    const sha = "a".repeat(64);

    expect(
      iconChoice({
        kind: "custom",
        sha256: sha,
        mime: "image/png",
        url: `portolano-icon://localhost/${sha}.png`,
      }),
    ).toEqual({ source: "custom", fileName: `${sha}.png` });
  });

  it("shows everything when nothing filters", async () => {
    const index = await sampleTree();

    expect(visibleIds(index, "all", null)).toBeNull();
  });

  it("shows the nodes a filter keeps and the folders that hold them", async () => {
    const index = await sampleTree();

    expect(names(index, visibleIds(index, "credentials", null))).toEqual([
      "Clienti",
      "Admin ACME",
      "Rete",
      "deploy",
    ]);
  });

  it("keeps only the nodes the search found among those of the filter", async () => {
    const index = await sampleTree();
    const found = new Set([await idOf("web01"), await idOf("bastion")]);

    expect(names(index, visibleIds(index, "favorites", found))).toEqual([
      "Clienti",
      "ACME",
      "web01",
    ]);
  });

  it("never shows a folder for its own name", async () => {
    const index = await sampleTree();

    expect(visibleIds(index, "all", new Set([await idOf("ACME")]))?.size).toBe(0);
  });

  it("puts a new node into the selected folder, or next to the selected node", async () => {
    const index = await sampleTree();

    expect(parentForNew(index, await idOf("ACME"))).toBe(await idOf("ACME"));
    expect(parentForNew(index, await idOf("web01"))).toBe(await idOf("ACME"));
    expect(parentForNew(index, await idOf("vnc-legacy"))).toBeNull();
    expect(parentForNew(index, null)).toBeNull();
  });

  it("lists the folders that hold a node, from its parent up", async () => {
    const index = await sampleTree();

    expect(ancestorsOf(index, await idOf("web01"))).toEqual([
      await idOf("ACME"),
      await idOf("Clienti"),
    ]);
  });

  it("writes a path as the interface does, keeping escaped slashes in the names", () => {
    expect(readablePath("/Clienti/ACME\\/Nord/web01")).toBe("Clienti / ACME/Nord / web01");
  });

  it("moves a node among the siblings shown only, while a filter hides some", async () => {
    const index = await sampleTree();
    const shown = new Set([await idOf("Clienti"), await idOf("ts01"), await idOf("ACME")]);

    expect(siblingPlacement(index, await idOf("ts01"), -1, shown)).toEqual({
      kind: "before",
      id: await idOf("ACME"),
    });
  });
});
