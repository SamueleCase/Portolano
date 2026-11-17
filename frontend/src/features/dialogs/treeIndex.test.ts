import { describe, expect, it } from "vitest";

import type { TreeNode_Serialize } from "../../bindings";
import { credentialsNamed, indexedNodes } from "./treeIndex";

let lastId = 0;

/** A node of the tree with an id of its own: names may repeat. */
function node(name: string, kind: TreeNode_Serialize["kind"], children: TreeNode_Serialize[] = []) {
  lastId += 1;
  return {
    id: `nodo-${lastId}`,
    name,
    kind,
    protocol: null,
    summary: "",
    color: null,
    icon: null,
    favorite: false,
    tags: [],
    secrets: "none",
    descendants: 0,
    children,
  } satisfies TreeNode_Serialize;
}

function entries(...nodes: TreeNode_Serialize[]) {
  return indexedNodes({ name: "Prova", nodes });
}

describe("credentialsNamed", () => {
  it("finds the credentials with exactly the name, in the whole document", () => {
    const tree = entries(
      node("Clienti", "folder", [node("deploy", "credential")]),
      node("Deploy", "credential"),
    );

    expect(credentialsNamed(tree, "deploy").map((entry) => entry.path)).toEqual([
      "Clienti / deploy",
    ]);
  });

  it("falls back to the names that differ only by case", () => {
    const tree = entries(node("Rete", "folder", [node("Deploy", "credential")]));

    expect(credentialsNamed(tree, "DEPLOY").map((entry) => entry.path)).toEqual(["Rete / Deploy"]);
  });

  it("returns every credential of an ambiguous name", () => {
    const tree = entries(
      node("A", "folder", [node("deploy", "credential")]),
      node("B", "folder", [node("deploy", "credential")]),
    );

    expect(credentialsNamed(tree, "deploy")).toHaveLength(2);
  });

  it("ignores the nodes that are not credentials", () => {
    const tree = entries(node("deploy", "folder"));

    expect(credentialsNamed(tree, "deploy")).toEqual([]);
  });
});

describe("indexedNodes", () => {
  it("gives each node the names from the root, in tree order", () => {
    const tree = entries(
      node("Clienti", "folder", [node("ACME", "folder", [node("web01", "connection")])]),
    );

    expect(tree.map((entry) => entry.path)).toEqual([
      "Clienti",
      "Clienti / ACME",
      "Clienti / ACME / web01",
    ]);
  });
});
