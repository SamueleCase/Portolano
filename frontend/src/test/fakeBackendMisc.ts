import type {
  ConnectPreview,
  ImportAnswerInput_Deserialize,
  ImportOptionsInput_Deserialize,
  ImportPreviewView_Serialize,
  ImportResult,
  Issue_Serialize,
  LoginRequest,
  SecretRequestView_Serialize,
} from "../bindings";
import {
  type FakeBackendState,
  blankNode,
  childrenOf,
  fail,
  findNode,
  openDocument,
  pathOf,
  requireUnlockedVault,
  walk,
} from "./fakeBackendModel";
import { traceOf } from "./fakeBackendViews";

/** Validation, import and the connection plan, simplified but with the same shapes and rules. */

export function validate(state: FakeBackendState): Issue_Serialize[] {
  const doc = openDocument(state);
  const issues: Issue_Serialize[] = [];
  for (const { node } of walk(doc.nodes)) {
    if (node.kind === "unsupported") {
      issues.push({
        severity: "info",
        node: node.id,
        code: "unsupported_node",
        message: `il nodo '${node.name}' non è supportato`,
      });
    }
    if (node.kind === "connection" && (node.host ?? "").trim() === "") {
      issues.push({
        severity: "error",
        node: node.id,
        code: "empty_host",
        message: `la connessione '${node.name}' non ha un host`,
      });
    }
    if (node.kind !== "unsupported") {
      const end = node.kind === "credential" ? null : traceOf(doc, node).end;
      if (end?.kind === "name_not_found") {
        issues.push({
          severity: "error",
          node: node.id,
          code: "by_name_unresolved",
          message: `nessuna credenziale si chiama '${end.name}'`,
        });
      }
    }
    if (!state.vault.locked) {
      for (const [, present] of Object.entries(node.secrets)) {
        if (!present) {
          issues.push({
            severity: "warning",
            node: node.id,
            code: "missing_secret",
            message: `un segreto del nodo '${node.name}' non è nel vault`,
          });
        }
      }
    }
  }
  if (state.vault.locked) {
    issues.push({
      severity: "info",
      node: null,
      code: "vault_locked",
      message: "il vault è bloccato: la presenza dei segreti non è stata controllata",
    });
  }
  return issues;
}

/** The fake file to import: two connections, one with a password to ask. */
const IMPORTED = [
  { name: "web10", protocol: "ssh", host: "198.51.100.10", username: "deploy", asks: true },
  { name: "dc10", protocol: "rdp", host: "198.51.100.20", username: null, asks: false },
];

function requests(): SecretRequestView_Serialize[] {
  return IMPORTED.filter((item) => item.asks).map((item) => ({
    nodePath: `Importati/${item.name}`,
    username: item.username,
    field: { kind: "password" },
  }));
}

function result(
  state: FakeBackendState,
  root: string,
  written: number,
  dryRun: boolean,
): ImportResult {
  const file = state.chosen.importFile ?? "";
  return {
    report: {
      importer: "royal",
      source_file: file.split("/").pop() ?? file,
      document_name: "Importati",
      imported_at: "2026-11-10T10:00:00Z",
      counts: {
        RoyalSSHConnection: {
          source: 1,
          imported: written > 0 ? 1 : 0,
          partial: written > 0 ? 0 : 1,
          unsupported: 0,
          skipped: 0,
        },
        RoyalRDSConnection: { source: 1, imported: 1, partial: 0, unsupported: 0, skipped: 0 },
      },
      items:
        written > 0
          ? []
          : [
              {
                source_type: "RoyalSSHConnection",
                source_id: "royal-web10",
                path: "Importati/web10",
                status: "partial",
                reasons: ["password non inserita"],
              },
            ],
      unmapped_fields: {},
      warnings: [],
    },
    root,
    inserted: IMPORTED.length + 1,
    updated: 0,
    secrets_written: written,
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
  openDocument(state);
  if (!state.chosen.importFile) fail("notFound", "nessun file da importare scelto");
  childrenOf(openDocument(state), options.into ?? null);
  return {
    result: result(state, "00000000-0000-4000-8000-0000000000ff", 0, true),
    requests: requests(),
  };
}

export function importApply(
  state: FakeBackendState,
  options: ImportOptionsInput_Deserialize,
  answers: ImportAnswerInput_Deserialize[],
): ImportResult {
  const doc = openDocument(state);
  if (!state.chosen.importFile) fail("notFound", "nessun file da importare scelto");
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
  const written = answers.filter((answer) => answer.value != null).length;
  if (written > 0) requireUnlockedVault(state);
  const target = childrenOf(doc, options.into ?? null);
  const root = blankNode(state, "folder", "Importati");
  for (const item of IMPORTED) {
    const node = blankNode(state, "connection", item.name);
    Object.assign(node, { protocol: item.protocol, host: item.host });
    node.source = {
      system: "royal",
      sourceType: item.protocol === "ssh" ? "RoyalSSHConnection" : "RoyalRDSConnection",
      sourceId: `royal-${item.name}`,
      file: state.chosen.importFile ?? null,
    };
    if (
      item.asks &&
      answers.some((answer) => answer.nodePath.endsWith(item.name) && answer.value != null)
    ) {
      node.credential = { mode: "inline", username: item.username, domain: null };
      node.secrets["password"] = true;
    }
    root.children.push(node);
  }
  target.push(root);
  state.lastImport = true;
  return result(state, root.id, written, false);
}

export function saveImportReport(state: FakeBackendState) {
  if (!state.lastImport) fail("notFound", "nessun import da esportare");
  const path = state.dialogs.report;
  return path === null ? null : { fileName: path.split("/").pop() ?? path };
}

export function loginRequest(state: FakeBackendState, id: string): LoginRequest | null {
  const doc = openDocument(state);
  const { node } = findNode(doc, id);
  if (node.kind !== "connection") fail("connect", `'${pathOf(doc, id)}' non è una connessione`);
  if (traceOf(doc, node).end.kind !== "prompt") return null;
  return {
    node: id,
    path: pathOf(doc, id),
    protocol: node.protocol ?? "",
    ask_domain: node.protocol === "rdp",
  };
}

export function connectPreview(state: FakeBackendState, id: string): ConnectPreview {
  const doc = openDocument(state);
  const { node } = findNode(doc, id);
  if (node.kind !== "connection") fail("connect", `'${pathOf(doc, id)}' non è una connessione`);
  const port =
    node.port === null ? "" : node.protocol === "ssh" ? ` -p ${node.port}` : `:${node.port}`;
  const command =
    node.protocol === "rdp"
      ? `/opt/homebrew/bin/sdl-freerdp /v:${node.host ?? ""}${port} /from-stdin:force < <vault:password>`
      : `/usr/bin/ssh${port} -- ${node.host ?? ""}`;
  const warnings =
    traceOf(doc, node).end.kind === "prompt" ? ["le credenziali si chiedono all'avvio"] : [];
  return { command, warnings };
}
