import { describe, expect, it } from "vitest";

import {
  commands,
  type ImportAnswerInput_Deserialize,
  type ImportOptionsInput_Deserialize,
  type SecretRequestView_Serialize,
} from "../bindings";
import { initialFakeState, installFakeBackend } from "./fakeBackend";
import { ok, refused, unlocked } from "./fakeBackendFixtures";
import { IMPORT_FILE, UNREADABLE_IMPORT_FILE } from "./fakeBackendImport";

/** The import of the fake backend, with the rules of `portolano-ops` (SPEC §8.4). */

const DEFAULTS: ImportOptionsInput_Deserialize = { into: null, onConflict: "fail" };

async function chosen(path = IMPORT_FILE, vaultLocked = false) {
  const state = vaultLocked ? installFakeBackend(initialFakeState()) : unlocked();
  state.dialogs.importFile = path;
  ok(await commands.chooseImportFile());
  return state;
}

function answers(
  requests: SecretRequestView_Serialize[],
  value: (index: number) => string | null,
): ImportAnswerInput_Deserialize[] {
  return requests.map((request, index) => ({
    nodePath: request.nodePath,
    field: request.field,
    value: value(index),
  }));
}

async function requests(options = DEFAULTS) {
  return ok(await commands.importPreview(options)).requests;
}

/** The imported nodes of the tree with their source ids, by name. */
async function importedSources(): Promise<Record<string, string | null>> {
  const found: Record<string, string | null> = {};
  const queue = [...ok(await commands.tree()).nodes];
  for (let node = queue.shift(); node; node = queue.shift()) {
    if (node.sourceId !== null) found[node.name] = node.sourceId;
    queue.push(...node.children);
  }
  return found;
}

describe("fake import", () => {
  it("previews three passwords to ask, in tree order, without writing", async () => {
    await chosen();

    const preview = ok(await commands.importPreview(DEFAULTS));

    expect(preview.requests.map((request) => request.nodePath)).toEqual([
      "Ufficio/Produzione/web10",
      "Ufficio/Produzione/db10",
      "Ufficio/Admin ufficio",
    ]);
    expect(preview.result.dry_run).toBe(true);
    expect(await importedSources()).toEqual({ "vnc-legacy": "royal-vnc" });
  });

  it("fails to read the unreadable file with an I/O error", async () => {
    await chosen(UNREADABLE_IMPORT_FILE);

    const error = refused(await commands.importPreview(DEFAULTS));

    expect(error.code).toBe("io");
  });

  it("makes a node partial when its password is skipped", async () => {
    await chosen();
    const asked = await requests();

    const result = ok(
      await commands.importApply(
        DEFAULTS,
        answers(asked, (index) => (index === 0 ? null : "sentinel-fake")),
      ),
    );

    const web10 = result.report.items.find((item) => item.path === "Ufficio/Produzione/web10");
    expect(web10?.status).toBe("partial");
    expect(result.secrets_written).toBe(2);
  });

  it("gives the imported nodes their Royal ids", async () => {
    await chosen();
    const asked = await requests();

    const result = ok(
      await commands.importApply(
        DEFAULTS,
        answers(asked, () => null),
      ),
    );

    const sources = await importedSources();
    expect(sources["Ufficio"]).toBe(result.root);
  });

  it("refuses to import again with the fail policy", async () => {
    await chosen();
    ok(
      await commands.importApply(
        DEFAULTS,
        answers(await requests(), () => null),
      ),
    );

    const error = refused(await commands.importPreview(DEFAULTS));

    expect(error.code).toBe("invalid");
  });

  it("imports again as new nodes with new ids keeping the source ids", async () => {
    await chosen();
    const first = ok(
      await commands.importApply(
        DEFAULTS,
        answers(await requests(), () => null),
      ),
    );
    const options: ImportOptionsInput_Deserialize = { ...DEFAULTS, onConflict: "newIds" };

    const again = ok(
      await commands.importApply(
        options,
        answers(await requests(options), () => null),
      ),
    );

    expect(again.root).not.toBe(first.root);
    expect(again.inserted).toBe(first.inserted);
  });

  it("updates the existing nodes in place with the update policy", async () => {
    await chosen();
    const first = ok(
      await commands.importApply(
        DEFAULTS,
        answers(await requests(), () => null),
      ),
    );
    const options: ImportOptionsInput_Deserialize = { ...DEFAULTS, onConflict: "update" };

    const again = ok(
      await commands.importApply(
        options,
        answers(await requests(options), () => null),
      ),
    );

    expect(again.root).toBe(first.root);
    expect([again.inserted, again.updated]).toEqual([0, first.inserted]);
  });

  it("skips the trash unless it is included", async () => {
    await chosen();

    const skipped = ok(await commands.importPreview(DEFAULTS)).result.report.counts;
    const included = ok(await commands.importPreview({ ...DEFAULTS, includeTrash: true })).result
      .report.counts;

    expect(skipped["RoyalTrash"]?.skipped).toBe(1);
    expect(included["RoyalTrash"]?.skipped).toBe(0);
  });

  it("refuses the typed passwords while the vault is locked", async () => {
    await chosen(IMPORT_FILE, true);

    const error = refused(
      await commands.importApply(
        DEFAULTS,
        answers(await requests(), () => "sentinel-fake"),
      ),
    );

    expect(error.code).toBe("vaultLocked");
  });
});
