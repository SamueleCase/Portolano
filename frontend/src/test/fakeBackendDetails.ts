import type { NodeDetailsView_Serialize, SecretSlot, SecretView, TraceEnd } from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeNode,
  type SlotKey,
  findNode,
  isLocked,
  pathOf,
} from "./fakeBackendModel";
import { DEFAULT_PORTS, iconView, specView, summaryOf, traceOf, usageOf } from "./fakeBackendViews";

/** The detail of a node, as `node_details`: without secrets, with their states. */

export function slotOf(key: SlotKey): SecretSlot {
  switch (key) {
    case "password":
      return "password";
    case "passphrase":
      return "passphrase";
    case "privateKey":
      return "private_key";
    default:
      return { property: key.slice("property:".length) };
  }
}

function secretViews(state: FakeBackendState, node: FakeNode): SecretView[] {
  return Object.entries(node.secrets).map(([key, present]) => ({
    slot: slotOf(key),
    state: isLocked(state) ? "vaultLocked" : present ? "present" : "absent",
  }));
}

export function detailsView(
  state: FakeBackendState,
  doc: FakeDocument,
  id: string,
): NodeDetailsView_Serialize {
  const { node, parent } = findNode(doc, id);
  const trace = node.kind === "unsupported" ? null : traceOf(doc, node);
  const effective = trace === null ? null : effectiveOf(state, doc, trace.end);
  return {
    id,
    name: node.name,
    path: pathOf(doc, id),
    parent: parent?.id ?? null,
    kind: node.kind,
    summary: summaryOf(node, doc),
    description: node.description,
    notes: node.notes,
    color: node.color,
    icon: iconView(node.icon),
    favorite: node.favorite,
    tags: [...node.tags],
    properties: node.properties.map((property) => ({
      name: property.name,
      text: property.text,
      secret: property.text === null,
    })),
    created: node.created,
    modified: node.modified,
    connection:
      node.kind === "connection"
        ? {
            protocol: node.protocol ?? "",
            host: node.host ?? "",
            port: node.port,
            defaultPort: DEFAULT_PORTS[node.protocol ?? ""] ?? null,
            settings: structuredClone(node.settings),
          }
        : null,
    credentialSpec:
      node.kind === "folder" || node.kind === "connection" ? specView(node.credential) : null,
    credential:
      node.kind === "credential"
        ? { username: node.username, domain: node.domain, keyFile: node.keyFile }
        : null,
    effective,
    trace,
    secrets: secretViews(state, node),
    vaultLocked: isLocked(state) && touchesTheVault(node, effective),
    usage: usageOf(doc, node),
    source: node.source ? { ...node.source, importedAt: node.created } : null,
    extra: structuredClone(node.extra),
    unsupported: node.unsupported ? { ...node.unsupported } : null,
  };
}

function effectiveOf(
  state: FakeBackendState,
  doc: FakeDocument,
  end: TraceEnd,
): NodeDetailsView_Serialize["effective"] {
  // With the vault locked the states are those of an empty vault, as in the
  // app; `vaultLocked` of the detail tells the interface to show them so.
  const secretState = (present: boolean | undefined) =>
    present === undefined ? "not_set" : present && !isLocked(state) ? "present" : "absent";
  switch (end.kind) {
    case "root":
    case "none":
      return { kind: "resolved", credential: emptyCredential({ kind: "none" }) };
    case "prompt":
      return { kind: "resolved", credential: emptyCredential({ kind: "prompt" }) };
    case "inline": {
      const holder = findNode(doc, end.node.id).node;
      const spec = holder.credential;
      return {
        kind: "resolved",
        credential: {
          ...emptyCredential({ kind: "inline", node: end.node.id, path: end.node.path }),
          username: spec.mode === "inline" ? spec.username : null,
          domain: spec.mode === "inline" ? spec.domain : null,
          password: secretState(holder.secrets["password"]),
        },
      };
    }
    case "credential": {
      const credential = findNode(doc, end.node.id).node;
      return {
        kind: "resolved",
        credential: {
          origin: { kind: "credential", node: end.node.id, path: end.node.path },
          username: credential.username,
          domain: credential.domain,
          password: secretState(credential.secrets["password"]),
          private_key: credential.keyFile
            ? { kind: "file", path: credential.keyFile }
            : credential.secrets["privateKey"] === undefined
              ? null
              : { kind: "vault", state: secretState(credential.secrets["privateKey"]) },
          passphrase: secretState(credential.secrets["passphrase"]),
        },
      };
    }
    case "name_not_found":
      return { kind: "unresolved", reason: `nessun nodo corrisponde a '${end.name}'` };
    case "name_ambiguous":
      return {
        kind: "unresolved",
        reason: `'${end.name}' è ambiguo: corrisponde a ${end.candidates.length} nodi`,
      };
    case "dangling":
      return { kind: "unresolved", reason: `il nodo ${end.target} non esiste` };
    case "not_a_credential":
      return { kind: "unresolved", reason: `il nodo ${end.node.path} non è una credenziale` };
  }
}

/** Whether the detail of `node` asks the vault: its own secrets or those of its credential. */
function touchesTheVault(
  node: FakeNode,
  effective: NodeDetailsView_Serialize["effective"],
): boolean {
  if (Object.keys(node.secrets).length > 0) return true;
  if (effective?.kind !== "resolved") return false;
  const { password, passphrase, private_key: key } = effective.credential;
  return password !== "not_set" || passphrase !== "not_set" || key?.kind === "vault";
}

function emptyCredential(
  origin: Extract<
    NodeDetailsView_Serialize["effective"],
    { kind: "resolved" }
  >["credential"]["origin"],
) {
  return {
    origin,
    username: null,
    domain: null,
    password: "not_set" as const,
    private_key: null,
    passphrase: "not_set" as const,
  };
}
