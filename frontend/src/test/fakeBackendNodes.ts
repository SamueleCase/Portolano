import type {
  AddOutcome,
  CredentialChoice_Deserialize,
  DetailsInput_Deserialize,
  DuplicateOutcome,
  EditOutcome,
  MoveOutcome,
  NewNodeInput_Deserialize,
  NodeEditInput_Deserialize,
  PlacementInput,
  StagedSecret,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeNode,
  type FakeSpec,
  blankNode,
  checkName,
  childrenOf,
  fail,
  failField,
  findNode,
  newId,
  openDocument,
  pathOf,
  requireUnlockedVault,
  subtreeIds,
  walk,
} from "./fakeBackendModel";

/** The changes to the nodes, with the rules of `portolano-ops`. */

const PROTOCOLS = ["ssh", "rdp"];

function specOf(state: FakeBackendState, choice: CredentialChoice_Deserialize): FakeSpec {
  switch (choice.mode) {
    case "inherit":
    case "none":
    case "prompt":
      return { mode: choice.mode };
    case "byName":
      return { mode: "byName", name: choice.name };
    case "inline":
      return { mode: "inline", username: choice.username ?? null, domain: choice.domain ?? null };
    case "byId": {
      const target = findNode(openDocument(state), choice.id).node;
      if (target.kind !== "credential") {
        failField(
          "credential",
          `'${pathOf(openDocument(state), target.id)}' non è una credenziale`,
        );
      }
      return { mode: "byId", id: choice.id };
    }
  }
}

function applyDetails(node: FakeNode, details: DetailsInput_Deserialize | undefined): void {
  if (!details) return;
  const blankIsNull = (value: string) => (value.trim() === "" ? null : value);
  if (details.description != null) node.description = blankIsNull(details.description);
  if (details.notes != null) node.notes = blankIsNull(details.notes);
  if (details.color != null) {
    if (details.color !== "" && !/^#[0-9a-fA-F]{6}$/.test(details.color)) {
      failField("color", `colore non valido '${details.color}': atteso #RRGGBB`);
    }
    node.color = blankIsNull(details.color);
  }
  if (details.favorite != null) node.favorite = details.favorite;
  if (details.tags != null) node.tags = [...details.tags];
  if (details.icon != null) {
    node.icon = details.icon.kind === "none" ? null : { ...details.icon };
  }
  const removed = new Set(details.removeSecretProperties ?? []);
  if (details.properties != null) {
    node.properties = [
      ...node.properties.filter((property) => property.text === null),
      ...details.properties.map((property) => ({ name: property.name, text: property.value })),
    ];
  }
  node.properties = node.properties.filter(
    (property) => !(property.text === null && removed.has(property.name)),
  );
  for (const name of removed) delete node.secrets[`property:${name}`];
}

function checkConnection(protocol: string, host: string, port: number | null): void {
  if (!PROTOCOLS.includes(protocol)) {
    failField(
      "protocol",
      `protocollo sconosciuto '${protocol}' (disponibili: ${PROTOCOLS.join(", ")})`,
    );
  }
  if (host.trim() === "") failField("host", "la connessione deve avere un host");
  if (port === 0) failField("port", "porta non valida: 0");
}

/** A password set on a folder or a connection: it needs an inline credential. */
function setInlinePassword(node: FakeNode): void {
  if (node.credential.mode === "inherit") {
    node.credential = { mode: "inline", username: null, domain: null };
  }
  if (node.credential.mode !== "inline") {
    failField("password", `la password non è applicabile a '${node.name}'`);
  }
  node.secrets["password"] = true;
}

export function addNode(state: FakeBackendState, input: NewNodeInput_Deserialize): AddOutcome {
  const doc = openDocument(state);
  const siblings = childrenOf(doc, input.parent ?? null);
  checkName(siblings, input.name);
  const secrets = input.secrets;
  if ((secrets?.password != null || (secrets?.properties?.length ?? 0) > 0) && state.vault.locked) {
    requireUnlockedVault(state);
  }
  const kind = input.kind;
  const node = blankNode(state, kind.type, input.name);
  switch (kind.type) {
    case "folder":
      node.credential = specOf(state, kind.credential);
      if (secrets?.password != null) setInlinePassword(node);
      break;
    case "connection":
      checkConnection(kind.protocol, kind.host, kind.port ?? null);
      Object.assign(node, { protocol: kind.protocol, host: kind.host, port: kind.port ?? null });
      node.settings = structuredClone(kind.settings);
      node.credential = specOf(state, kind.credential);
      if (secrets?.password != null) setInlinePassword(node);
      break;
    case "credential":
      if (kind.keyFile && secrets?.privateKey != null) {
        failField(
          "private_key",
          "indica la chiave privata come file oppure nel vault, non entrambe",
        );
      }
      Object.assign(node, { username: kind.username, domain: kind.domain, keyFile: kind.keyFile });
      if (secrets?.password != null) node.secrets["password"] = true;
      if (secrets?.passphrase != null) node.secrets["passphrase"] = true;
      if (secrets?.privateKey != null) node.secrets["privateKey"] = true;
      break;
  }
  if (kind.type !== "credential" && (secrets?.passphrase != null || secrets?.privateKey != null)) {
    failField("passphrase", `la passphrase non è applicabile a '${input.name}'`);
  }
  for (const property of secrets?.properties ?? []) {
    node.properties.push({ name: property.name, text: null });
    node.secrets[`property:${property.name}`] = true;
  }
  applyDetails(node, input.details);
  siblings.push(node);
  const written = Object.keys(node.secrets).length;
  return { id: node.id, path: pathOf(doc, node.id), secrets_written: written };
}

function stage(
  state: FakeBackendState,
  node: FakeNode,
  key: string,
  change: StagedSecret | null | undefined,
): number {
  if (!change) return 0;
  if (change.action === "clear") {
    if (node.secrets[key] === undefined) {
      failField(
        key === "privateKey" ? "private_key" : key,
        `il segreto di '${node.name}' non è impostato`,
      );
    }
    delete node.secrets[key];
    return 0;
  }
  requireUnlockedVault(state);
  node.secrets[key] = true;
  return 1;
}

export function editNode(
  state: FakeBackendState,
  id: string,
  edit: NodeEditInput_Deserialize,
): EditOutcome {
  const doc = openDocument(state);
  const { node: original, parent } = findNode(doc, id);
  // The backend works on a copy and keeps the node unchanged on error.
  const node = structuredClone(original);
  if (edit.name != null) checkName(parent ? parent.children : doc.nodes, edit.name, id);
  if (edit.name != null) node.name = edit.name;
  applyDetails(node, edit.details);
  if (edit.host != null || edit.port != null || edit.defaultPort || edit.settings != null) {
    if (node.kind !== "connection")
      failField("host", `il campo 'host' non si può modificare su '${node.name}'`);
    if (edit.host != null) node.host = edit.host;
    if (edit.port != null) node.port = edit.port;
    if (edit.defaultPort) node.port = null;
    if (edit.settings != null) node.settings = structuredClone(edit.settings);
    checkConnection(node.protocol ?? "", node.host ?? "", node.port);
  }
  if (edit.credential != null) {
    if (node.kind === "credential" || node.kind === "unsupported") {
      failField("credential", `il campo 'credential' non si può modificare su '${node.name}'`);
    }
    node.credential = specOf(state, edit.credential);
    if (node.credential.mode !== "inline") delete node.secrets["password"];
  }
  if (node.kind === "credential") {
    if (edit.username != null) node.username = edit.username || null;
    if (edit.domain != null) node.domain = edit.domain || null;
    if (edit.keyFile != null) node.keyFile = edit.keyFile || null;
  }
  const secrets = edit.secrets;
  let written = 0;
  if (secrets?.password?.action === "set" && node.kind !== "credential") setInlinePassword(node);
  written += stage(state, node, "password", secrets?.password);
  written += stage(state, node, "passphrase", secrets?.passphrase);
  written += stage(state, node, "privateKey", secrets?.privateKey);
  if (
    node.keyFile &&
    node.secrets["privateKey"] !== undefined &&
    secrets?.privateKey?.action === "set"
  ) {
    node.keyFile = null;
  }
  for (const property of secrets?.properties ?? []) {
    requireUnlockedVault(state);
    node.properties = node.properties.filter((existing) => existing.name !== property.name);
    node.properties.push({ name: property.name, text: null });
    node.secrets[`property:${property.name}`] = true;
    written += 1;
  }
  Object.assign(original, node);
  return {
    path: pathOf(doc, id),
    secrets_written: written,
    secrets_deleted: 0,
    cleanup_failures: [],
  };
}

export function moveNode(
  state: FakeBackendState,
  id: string,
  destination: string | null,
  placement: PlacementInput,
): MoveOutcome {
  const doc = openDocument(state);
  const { node, parent } = findNode(doc, id);
  if (destination !== null && subtreeIds(node).has(destination)) {
    fail("invalid", `impossibile spostare il nodo ${id} dentro sé stesso o un suo discendente`);
  }
  const target = childrenOf(doc, destination);
  checkName(target, node.name, id);
  const source = parent ? parent.children : doc.nodes;
  source.splice(source.indexOf(node), 1);
  const index = (() => {
    switch (placement.kind) {
      case "end":
        return target.length;
      case "index":
        return Math.min(placement.index, target.length);
      case "before":
      case "after": {
        const sibling = target.findIndex((child) => child.id === placement.id);
        if (sibling < 0) fail("invalid", `'${placement.id}' non è un elemento della destinazione`);
        return sibling + (placement.kind === "after" ? 1 : 0);
      }
    }
  })();
  target.splice(index, 0, node);
  return { path: pathOf(doc, id) };
}

export function duplicateNode(state: FakeBackendState, id: string): DuplicateOutcome {
  const doc = openDocument(state);
  const { node, parent } = findNode(doc, id);
  const siblings = parent ? parent.children : doc.nodes;
  const copy = structuredClone(node);
  const hasSecrets = [...walk([copy])].some(
    ({ node: inner }) => Object.keys(inner.secrets).length > 0,
  );
  if (hasSecrets) requireUnlockedVault(state);
  let copied = 0;
  for (const { node: inner } of walk([copy])) {
    inner.id = newId(state);
    copied += Object.values(inner.secrets).filter(Boolean).length;
  }
  let name = `${node.name} (copia)`;
  for (let counter = 2; siblings.some((sibling) => sibling.name === name); counter += 1) {
    name = `${node.name} (copia ${counter})`;
  }
  copy.name = name;
  siblings.splice(siblings.indexOf(node) + 1, 0, copy);
  const missing = [...walk([copy])].reduce(
    (count, { node: inner }) =>
      count + Object.values(inner.secrets).filter((present) => !present).length,
    0,
  );
  return {
    id: copy.id,
    path: pathOf(doc, copy.id),
    secrets_copied: copied,
    secrets_missing: missing,
  };
}
