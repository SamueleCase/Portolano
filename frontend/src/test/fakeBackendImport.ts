import type {
  ImportAnswerInput_Deserialize,
  ImportOptionsInput_Deserialize,
  ImportPreviewView_Serialize,
  ImportResult,
  ItemStatus,
  ReportItem,
  SecretRequestView_Serialize,
  TypeCounts,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeNode,
  blankNode,
  childrenOf,
  fail,
  fileName,
  newId,
  openDocument,
  requireVaultFor,
  walk,
} from "./fakeBackendModel";
import { DOCUMENT, OBJECTS, type SourceObject, pathOfObject } from "./fakeBackendImportSource";

/**
 * The import of the fake backend: one invented Royal TSX document, simplified
 * but with the shapes and the rules of `portolano-ops` (SPEC §8.3, §8.4):
 * three passwords to ask, a partial node whatever the answers (an icon over
 * 4 MiB, DEC-031.1), an unsupported node and the trash, skipped unless it is
 * included (DEC-016.8). As in the core, the id of a node is its Royal id,
 * except with `newIds`.
 */

/** An answer of the import dialog for the tests: the fake Royal document. */
export const IMPORT_FILE = "/Users/utente-di-prova/Documenti/ufficio.rtsz";

/** The answer of the import dialog that cannot be read: its preview fails with `io`. */
export const UNREADABLE_IMPORT_FILE = "/Users/utente-di-prova/Documenti/illeggibile.rtsz";

const SKIPPED_PASSWORD =
  "password Royal cifrata non importata: impostala con `portolano secret set`";
const IN_TRASH = "nel cestino di Royal TSX: non importato";

function requests(): SecretRequestView_Serialize[] {
  return OBJECTS.filter((object) => object.asksPassword).map((object) => ({
    nodePath: pathOfObject(object),
    username: object.username ?? null,
    field: { kind: "password" },
  }));
}

/** The objects that become nodes, with the outcome of each. */
function outcomes(includeTrash: boolean, answered: ReadonlySet<string>) {
  return OBJECTS.map((object) => {
    const reasons = [...(object.reasons ?? [])];
    let status: ItemStatus = object.kind === "unsupported" ? "unsupported" : "imported";
    if (object.inTrash && !includeTrash) {
      status = "skipped";
      reasons.push(IN_TRASH);
    } else if (object.asksPassword && !answered.has(pathOfObject(object))) {
      reasons.push(SKIPPED_PASSWORD);
    }
    if (status === "imported" && reasons.length > 0) status = "partial";
    return { object, status, reasons };
  });
}

function report(state: FakeBackendState, includeTrash: boolean, answered: ReadonlySet<string>) {
  const counts: Record<string, TypeCounts> = {};
  const items: ReportItem[] = [];
  for (const { object, status, reasons } of outcomes(includeTrash, answered)) {
    const count = (counts[object.sourceType] ??= {
      source: 0,
      imported: 0,
      partial: 0,
      unsupported: 0,
      skipped: 0,
    });
    count.source += 1;
    count[status] += 1;
    if (status !== "imported") {
      items.push({
        source_type: object.sourceType,
        source_id: object.sourceId,
        path: pathOfObject(object),
        status,
        reasons,
      });
    }
  }
  return {
    importer: "royal",
    source_file: fileName(state.chosen.importFile ?? ""),
    document_name: "Ufficio",
    imported_at: "2026-11-10T10:00:00Z",
    counts,
    items,
    unmapped_fields: { RoyalRDSConnection: { CustomImage: 1 } },
    warnings: ["documento protetto da password: la password del documento non è stata importata"],
  };
}

/** The file chosen in the import dialog, read: an unreadable one fails as the Rust command. */
function chosenFile(state: FakeBackendState): string {
  const path = state.chosen.importFile ?? fail("notFound", "nessun file da importare scelto");
  if (path === UNREADABLE_IMPORT_FILE) {
    fail("io", `errore di I/O su '${path}': Permission denied (os error 13)`);
  }
  return path;
}

/** What the import does with the document: the ids it gives and the nodes it replaces. */
function plan(state: FakeBackendState, options: ImportOptionsInput_Deserialize) {
  const doc = openDocument(state);
  childrenOf(doc, options.into ?? null);
  const policy = options.onConflict ?? "fail";
  const existing = new Map([...walk(doc.nodes)].map(({ node }) => [node.id, node]));
  const included = OBJECTS.filter((object) => options.includeTrash || !object.inTrash);
  const conflicts = policy === "newIds" ? [] : included.filter((o) => existing.has(o.sourceId));
  if (policy === "fail" && conflicts.length > 0) {
    fail("invalid", `${conflicts.length} nodi dell'import esistono già nel documento`);
  }
  const ids = new Map(
    included.map((object) => [
      object.sourceId,
      policy === "newIds" ? newId(state) : object.sourceId,
    ]),
  );
  return { doc, existing, included, ids, updated: conflicts.length };
}

function result(
  state: FakeBackendState,
  options: ImportOptionsInput_Deserialize,
  answered: ReadonlySet<string>,
  planned: ReturnType<typeof plan>,
  dryRun: boolean,
): ImportResult {
  return {
    report: report(state, options.includeTrash ?? false, answered),
    root: planned.ids.get(DOCUMENT) ?? DOCUMENT,
    inserted: planned.included.length - planned.updated,
    updated: planned.updated,
    secrets_written: answered.size,
    icons_stored: 0,
    warnings: [],
    cleanup_failures: [],
    dry_run: dryRun,
  };
}

export function importPreview(
  state: FakeBackendState,
  options: ImportOptionsInput_Deserialize,
): ImportPreviewView_Serialize {
  chosenFile(state);
  const planned = plan(state, options);
  return { result: result(state, options, new Set(), planned, true), requests: requests() };
}

/** Puts `object` and its content in `siblings`, replacing the node of the same id if any. */
function insert(
  state: FakeBackendState,
  object: SourceObject,
  siblings: FakeNode[],
  planned: ReturnType<typeof plan>,
  answered: ReadonlySet<string>,
): void {
  const id = planned.ids.get(object.sourceId) ?? object.sourceId;
  const replaced = planned.existing.get(id);
  const node = replaced ?? blankNode(state, object.kind, object.name);
  Object.assign(node, {
    id,
    name: object.name,
    protocol: object.protocol ?? null,
    host: object.host ?? null,
    source: {
      system: "royal",
      sourceType: object.sourceType,
      sourceId: object.sourceId,
      file: fileName(state.chosen.importFile ?? ""),
    },
  });
  if (object.kind === "credential") node.username = object.username ?? null;
  if (object.asksPassword && answered.has(pathOfObject(object))) {
    if (object.kind === "connection") {
      node.credential = { mode: "inline", username: object.username ?? null, domain: null };
    }
    node.secrets["password"] = true;
  }
  if (!replaced) siblings.push(node);
  for (const child of planned.included.filter((other) => other.parent === object.sourceId)) {
    insert(state, child, node.children, planned, answered);
  }
}

export function importApply(
  state: FakeBackendState,
  options: ImportOptionsInput_Deserialize,
  answers: ImportAnswerInput_Deserialize[],
): ImportResult {
  chosenFile(state);
  const expected = requests();
  const mismatch =
    answers.length !== expected.length ||
    answers.some(
      (answer, index) =>
        answer.nodePath !== expected[index]?.nodePath ||
        answer.field.kind !== expected[index]?.field.kind,
    );
  if (mismatch) {
    fail(
      "invalid",
      "le password inserite non corrispondono più alle richieste dell'import: ripeti l'anteprima",
    );
  }
  const answered = new Set(
    answers.filter((answer) => answer.value != null).map((answer) => answer.nodePath),
  );
  requireVaultFor(state, answered.size > 0);
  const planned = plan(state, options);
  const root = OBJECTS.find((object) => object.sourceId === DOCUMENT);
  if (root) insert(state, root, childrenOf(planned.doc, options.into ?? null), planned, answered);
  state.lastImport = true;
  return result(state, options, answered, planned, false);
}

export function saveImportReport(state: FakeBackendState) {
  if (!state.lastImport) fail("notFound", "nessun import da esportare");
  const path = state.dialogs.report;
  return path === null ? null : { fileName: fileName(path) };
}
