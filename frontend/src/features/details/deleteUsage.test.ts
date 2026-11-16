import { describe, expect, it } from "vitest";

import { commands, type DeletePreview } from "../../bindings";
import { installFakeBackend } from "../../test/fakeBackend";
import { idOf, ok } from "../../test/fakeBackendFixtures";
import { indexTree } from "../tree/treeModel";
import { deleteUsage } from "./deleteUsage";

/** The folder Rete deleted, with what refers to its content. */
function folderPreview(references: DeletePreview["references"]): DeletePreview {
  return {
    id: "rete",
    path: "/Rete",
    node_type: { type: "folder" },
    children: 2,
    descendants: 2,
    secrets: 1,
    references,
    inherited: [],
  };
}

const reference = (target: string, targetPath: string) => ({
  from: "web01",
  from_path: "/Clienti/web01",
  target,
  target_path: targetPath,
});

describe("who uses what a deletion removes", () => {
  it("names a jump host it refers to with its name made visible", () => {
    const usage = deleteUsage(
      folderPreview([reference("bastion", "/Rete/bas\u001btion")]),
      undefined,
    );

    expect(usage.rows[0]?.relation).toBe("fa riferimento a bas\\u{1b}tion");
    expect(usage.otherReferences).toBe(true);
  });

  it("names the credential inside the deleted folder a node is linked to", async () => {
    installFakeBackend();
    const index = indexTree(ok(await commands.tree()));
    const deploy = await idOf("deploy");

    const usage = deleteUsage(folderPreview([reference(deploy, "/Rete/de‮ploy")]), index);

    expect(usage.rows[0]?.relation).toBe("collegata a de\\u{202e}ploy");
    expect(usage.credentialsUsed).toBe(true);
  });
});
