import type { EditOutcome, NodeEditInput_Deserialize, StagedSecret } from "../bindings";
import {
  type FakeBackendState,
  type FakeNode,
  type SlotKey,
  checkName,
  fail,
  failField,
  findNode,
  openDocument,
  pathOf,
  requireVaultFor,
  siblingsOf,
  slotField,
} from "./fakeBackendModel";
import {
  applyDetails,
  checkConnection,
  checkKeyFile,
  setInlinePassword,
  specOf,
} from "./fakeBackendNodes";

/**
 * The editor's changes (`edit_node_staged`): the same checks and fields of
 * the errors as `portolano-ops` and the app, on a copy of the node that
 * replaces it only on success.
 */
export function editNode(
  state: FakeBackendState,
  id: string,
  edit: NodeEditInput_Deserialize,
): EditOutcome {
  const doc = openDocument(state);
  const { node: original, parent } = findNode(doc, id);
  const node = structuredClone(original);
  checkKeyFile(edit.keyFile);
  if (edit.name != null) {
    checkName(siblingsOf(doc, parent), edit.name, id);
    node.name = edit.name;
  }
  applyDetails(state, node, edit.details);
  editTarget(node, edit);
  editLogin(state, node, edit);
  let written = 0;
  if (node.kind === "credential") {
    written += stage(node, "password", edit.secrets?.password);
    written += stage(node, "passphrase", edit.secrets?.passphrase);
    written += stage(node, "privateKey", edit.secrets?.privateKey);
    editKeyFile(node, edit);
  } else {
    refuseOnLogin(node, edit);
    if (edit.secrets?.password?.action === "set") setInlinePassword(node);
    written += stage(node, "password", edit.secrets?.password);
  }
  for (const property of edit.secrets?.properties ?? []) {
    node.properties = node.properties.filter((existing) => existing.name !== property.name);
    node.properties.push({ name: property.name, text: null });
    node.secrets[`property:${property.name}`] = true;
    written += 1;
  }
  const removed = Object.keys(original.secrets).filter((key) => !(key in node.secrets)).length;
  requireVaultFor(state, written > 0 || removed > 0);
  Object.assign(original, node);
  return {
    path: pathOf(doc, id),
    secrets_written: written,
    secrets_deleted: removed,
    cleanup_failures: [],
  };
}

/** Host, port and settings, only on a connection; the field of the error is the first given. */
function editTarget(node: FakeNode, edit: NodeEditInput_Deserialize): void {
  const field =
    edit.host != null
      ? "host"
      : edit.port != null || edit.defaultPort
        ? "port"
        : edit.settings != null
          ? "settings"
          : null;
  if (field === null) return;
  if (node.kind !== "connection") {
    failField(field, `il campo '${field}' non si può modificare su '${node.name}'`);
  }
  if (edit.host != null) node.host = edit.host;
  if (edit.port != null) node.port = edit.port;
  if (edit.defaultPort) node.port = null;
  if (edit.settings != null) node.settings = structuredClone(edit.settings);
  checkConnection(node.protocol ?? "", node.host ?? "", node.port);
}

/** The credential, or user and domain: fields of a credential, inline login of the others. */
function editLogin(state: FakeBackendState, node: FakeNode, edit: NodeEditInput_Deserialize): void {
  const login = edit.username != null || edit.domain != null;
  if (node.kind === "credential") {
    if (edit.credential != null) {
      failField("credential", `il campo 'credential' non si può modificare su '${node.name}'`);
    }
    if (edit.username != null) node.username = edit.username || null;
    if (edit.domain != null) node.domain = edit.domain || null;
    return;
  }
  if (edit.credential == null && !login) return;
  if (node.kind === "unsupported") {
    failField(
      edit.credential != null ? "credential" : "user",
      `il campo non si può modificare su '${node.name}'`,
    );
  }
  if (edit.credential != null && login) {
    failField("credential", `per '${node.name}' indica la credenziale oppure utente e dominio`);
  }
  if (edit.credential != null) {
    node.credential = specOf(state, edit.credential);
  } else {
    const current = node.credential.mode === "inline" ? node.credential : null;
    node.credential = {
      mode: "inline",
      username: edit.username != null ? edit.username || null : (current?.username ?? null),
      domain: edit.domain != null ? edit.domain || null : (current?.domain ?? null),
    };
  }
  if (node.credential.mode !== "inline") delete node.secrets["password"];
}

/** Passphrase, private key and key file belong to credentials only. */
function refuseOnLogin(node: FakeNode, edit: NodeEditInput_Deserialize): void {
  if (edit.secrets?.passphrase != null) {
    failField("passphrase", `la passphrase non è applicabile a '${node.name}'`);
  }
  if (edit.secrets?.privateKey != null) {
    failField("private_key", `la chiave privata non è applicabile a '${node.name}'`);
  }
  if (edit.keyFile != null) {
    failField("key_file", `il campo 'key_file' non si può modificare su '${node.name}'`);
  }
}

/** The key of a credential: in the vault or a file, never both. */
function editKeyFile(node: FakeNode, edit: NodeEditInput_Deserialize): void {
  const vaultKey = edit.secrets?.privateKey?.action === "set";
  if (vaultKey && edit.keyFile) {
    failField("private_key", "indica la chiave privata come file oppure nel vault, non entrambe");
  }
  if (vaultKey) node.keyFile = null;
  if (edit.keyFile != null) node.keyFile = edit.keyFile || null;
}

/** Applies a staged secret; returns how many secrets it writes. */
function stage(node: FakeNode, key: SlotKey, change: StagedSecret | null | undefined): number {
  if (!change) return 0;
  if (change.action === "clear") {
    if (node.secrets[key] === undefined) {
      fail("invalid", `il segreto di '${node.name}' non è impostato`, {
        kind: "invalid",
        field: slotField(key),
      });
    }
    delete node.secrets[key];
    return 0;
  }
  node.secrets[key] = true;
  return 1;
}
