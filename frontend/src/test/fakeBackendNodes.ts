import { isControl } from "../components/visibleText";
import type {
  AddOutcome,
  CredentialChoice_Deserialize,
  DetailsInput_Deserialize,
  MoveOutcome,
  NewNodeInput_Deserialize,
  PlacementInput,
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
  openDocument,
  pathOf,
  requireVaultFor,
  siblingsOf,
  subtreeIds,
} from "./fakeBackendModel";

/** Creating and moving nodes, and the rules the editor shares, as in `portolano-ops`. */

const PROTOCOLS = ["ssh", "rdp"];

export function specOf(state: FakeBackendState, choice: CredentialChoice_Deserialize): FakeSpec {
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
      const doc = openDocument(state);
      const target = findNode(doc, choice.id).node;
      if (target.kind !== "credential") {
        failField("credential", `'${pathOf(doc, target.id)}' non è una credenziale`);
      }
      return { mode: "byId", id: choice.id };
    }
  }
}

export function applyDetails(
  state: FakeBackendState,
  node: FakeNode,
  details: DetailsInput_Deserialize | undefined,
): void {
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
  const icon = details.icon;
  if (icon != null) {
    if (
      icon.kind === "custom" &&
      !state.customIcons.some((stored) => stored.kind === "custom" && stored.sha256 === icon.sha256)
    ) {
      failField("icon", "l'immagine scelta come icona non è fra quelle del documento");
    }
    node.icon = icon.kind === "none" ? null : { ...icon };
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

export function checkConnection(protocol: string, host: string, port: number | null): void {
  if (!PROTOCOLS.includes(protocol)) {
    failField(
      "protocol",
      `protocollo sconosciuto '${protocol}' (disponibili: ${PROTOCOLS.join(", ")})`,
    );
  }
  if (host.trim() === "") failField("host", "la connessione deve avere un host");
  if (port === 0) failField("port", "porta non valida: 0");
}

/** A password set on a folder or a connection: an inherited credential becomes inline. */
export function setInlinePassword(node: FakeNode): void {
  if (node.credential.mode === "inherit") {
    node.credential = { mode: "inline", username: null, domain: null };
  }
  if (node.credential.mode !== "inline") {
    failField("password", `la password non è applicabile a '${node.name}'`);
  }
  node.secrets["password"] = true;
}

/** Whether `text` holds a control character, as `char::is_control` of the backend. */
function hasControlCharacter(text: string): boolean {
  return [...text].some((char) => isControl(char.codePointAt(0) ?? 0));
}

/** A key file, as `check_key_file` of the app (DEC-031.4). */
export function checkKeyFile(path: string | null | undefined): void {
  if (!path) return;
  if (!(path.startsWith("/") || path.startsWith("~/")) || hasControlCharacter(path)) {
    failField("key_file", `file della chiave non valido: ${JSON.stringify(path)}`);
  }
}

export function addNode(state: FakeBackendState, input: NewNodeInput_Deserialize): AddOutcome {
  const doc = openDocument(state);
  if (input.name.trim() === "") failField("name", "il nome del nodo non può essere vuoto");
  const siblings = childrenOf(doc, input.parent ?? null);
  checkName(siblings, input.name);
  const secrets = input.secrets;
  const kind = input.kind;
  const node = blankNode(state, kind.type, input.name);
  if (kind.type !== "credential") {
    if (secrets?.passphrase != null) {
      failField("passphrase", `la passphrase non è applicabile a '${input.name}'`);
    }
    if (secrets?.privateKey != null) {
      failField("private_key", `la chiave privata non è applicabile a '${input.name}'`);
    }
  }
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
      checkKeyFile(kind.keyFile);
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
  for (const property of secrets?.properties ?? []) {
    node.properties.push({ name: property.name, text: null });
    node.secrets[`property:${property.name}`] = true;
  }
  applyDetails(state, node, input.details);
  const written = Object.keys(node.secrets).length;
  requireVaultFor(state, written > 0);
  siblings.push(node);
  return { id: node.id, path: pathOf(doc, node.id), secrets_written: written };
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
  // Every check happens on the final list before the node leaves its place,
  // so that a refused move changes nothing.
  const remaining = target.filter((child) => child.id !== id);
  const index = (() => {
    switch (placement.kind) {
      case "end":
        return remaining.length;
      case "index":
        return Math.min(placement.index, remaining.length);
      case "before":
      case "after": {
        const sibling = remaining.findIndex((child) => child.id === placement.id);
        if (sibling < 0) fail("invalid", `'${placement.id}' non è un elemento della destinazione`);
        return sibling + (placement.kind === "after" ? 1 : 0);
      }
    }
  })();
  const source = siblingsOf(doc, parent);
  source.splice(source.indexOf(node), 1);
  target.splice(index, 0, node);
  return { path: pathOf(doc, id) };
}
