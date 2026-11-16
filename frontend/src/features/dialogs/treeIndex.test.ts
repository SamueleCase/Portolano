import { describe, expect, it } from "vitest";

import type { TreeNode_Serialize } from "../../bindings";
import { readablePath } from "./nodePaths";
import { credentialsNamed, indexTree } from "./treeIndex";

function node(name: string, kind: TreeNode_Serialize["kind"], children: TreeNode_Serialize[] = []) {
  return {
    id: name,
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
  return indexTree({ name: "Prova", nodes });
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

describe("readablePath", () => {
  it("writes the names with their escaped slashes", () => {
    expect(readablePath("/Clienti/A\\/B/web\\\\01")).toBe("Clienti / A/B / web\\01");
  });
});
