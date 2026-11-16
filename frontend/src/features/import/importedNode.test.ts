import { describe, expect, it } from "vitest";

import type { TreeNode_Serialize, TreeView_Serialize } from "../../bindings";
import { importedNode } from "./importedNode";

function node(id: string, sourceId: string | null, children: TreeNode_Serialize[] = []) {
  const tree: TreeNode_Serialize = {
    id,
    name: id,
    kind: children.length > 0 ? "folder" : "connection",
    protocol: null,
    summary: "",
    color: null,
    icon: null,
    favorite: false,
    tags: [],
    secrets: "none",
    descendants: 0,
    sourceId,
    children,
  };
  return tree;
}

function tree(...nodes: TreeNode_Serialize[]): TreeView_Serialize {
  return { name: "Test", nodes };
}

describe("importedNode", () => {
  it("prefers the node under the folder of the import", () => {
    const twice = tree(
      node("first", "royal-root", [node("first-web", "royal-web")]),
      node("second", "royal-root", [node("second-web", "royal-web")]),
    );

    expect(importedNode(twice, "second", "royal-web")).toBe("second-web");
  });

  it("finds an updated node outside the folder of the import", () => {
    const moved = tree(node("root", "royal-root"), node("elsewhere", "royal-web"));

    expect(importedNode(moved, "root", "royal-web")).toBe("elsewhere");
  });

  it("finds nothing when the node left the document", () => {
    expect(importedNode(tree(node("root", "royal-root")), "root", "royal-web")).toBeNull();
  });
});
