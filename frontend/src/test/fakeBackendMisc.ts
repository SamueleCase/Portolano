import type { ConnectPreview, Issue_Serialize, LoginRequest } from "../bindings";
import {
  type FakeBackendState,
  fail,
  findNode,
  openDocument,
  pathOf,
  isLocked,
  walk,
} from "./fakeBackendModel";
import { traceOf } from "./fakeBackendViews";

/** Validation, import and the connection plan, simplified but with the same shapes and rules. */

export { importApply, importPreview, saveImportReport } from "./fakeBackendImport";

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
    if (!isLocked(state)) {
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
  if (isLocked(state)) {
    issues.push({
      severity: "info",
      node: null,
      code: "vault_locked",
      message: "il vault è bloccato: la presenza dei segreti non è stata controllata",
    });
  }
  return issues;
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
