import type {
  CredentialTrace_Serialize,
  CredentialUsage,
  IconView,
  NodeDetailsView_Serialize,
  NodeKindView,
  NodeRef,
  SecretSlot,
  SecretView,
  SecretsBadge,
  SpecView_Serialize,
  TraceEnd,
  TreeNode_Serialize,
  TreeView_Serialize,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeIcon,
  type FakeNode,
  type FakeSpec,
  type SlotKey,
  findNode,
  pathOf,
  walk,
} from "./fakeBackendModel";

/** The read-only views of the fake document: what `tree` and `node_details` return. */

const DEFAULT_PORTS: Record<string, number> = { ssh: 22, rdp: 3389 };

/** The one-line summary, as `inspect::summary` of the backend. */
export function summaryOf(node: FakeNode, doc: FakeDocument): string {
  switch (node.kind) {
    case "folder":
      return "";
    case "credential":
      return loginName(node.username, node.domain);
    case "unsupported":
      return `non supportato: ${node.unsupported?.sourceType ?? ""}`;
    case "connection": {
      const login = effectiveLogin(doc, node);
      const user = login ? `${login}@` : "";
      const port = node.port ?? DEFAULT_PORTS[node.protocol ?? ""];
      return `${user}${node.host ?? ""}${port === undefined ? "" : `:${port}`}`;
    }
  }
}

function loginName(username: string | null, domain: string | null): string {
  if (domain && username) return `${domain}\\${username}`;
  if (domain) return `${domain}\\`;
  return username ?? "";
}

function effectiveLogin(doc: FakeDocument, node: FakeNode): string {
  const end = traceOf(doc, node).end;
  if (end.kind === "inline") {
    const holder = findNode(doc, end.node.id).node;
    return holder.credential.mode === "inline" ? (holder.credential.username ?? "") : "";
  }
  if (end.kind === "credential") {
    const credential = findNode(doc, end.node.id).node;
    return loginName(credential.username, credential.domain);
  }
  return "";
}

/** The icon to show, as `IconView::of`: Royal names through the words of the backend. */
export function iconView(icon: FakeIcon | null): IconView | null {
  if (icon === null) return null;
  switch (icon.kind) {
    case "builtin":
      return { kind: "builtin", name: icon.name };
    case "royal":
      return /server/i.test(icon.name) ? { kind: "builtin", name: "server" } : null;
    case "custom": {
      const extension = icon.mime === "image/svg+xml" ? "svg" : (icon.mime.split("/")[1] ?? "png");
      return {
        kind: "custom",
        sha256: icon.sha256,
        mime: icon.mime,
        url: `portolano-icon://localhost/${icon.sha256}.${extension}`,
      };
    }
  }
}

/** The badge of the secrets of a node itself. */
export function badgeOf(state: FakeBackendState, node: FakeNode): SecretsBadge {
  const present = Object.values(node.secrets);
  if (present.length === 0) return "none";
  if (state.vault.locked) return "vaultLocked";
  return present.every(Boolean) ? "present" : "absent";
}

export function treeView(state: FakeBackendState, doc: FakeDocument): TreeView_Serialize {
  const toTree = (node: FakeNode): TreeNode_Serialize => {
    const children = node.children.map(toTree);
    return {
      id: node.id,
      name: node.name,
      kind: node.kind satisfies NodeKindView,
      protocol: node.protocol,
      summary: summaryOf(node, doc),
      color: node.color,
      icon: iconView(node.icon),
      favorite: node.favorite,
      tags: [...node.tags],
      secrets: badgeOf(state, node),
      descendants: children.reduce((count, child) => count + child.descendants + 1, 0),
      children,
    };
  };
  return { name: doc.name, nodes: doc.nodes.map(toTree) };
}

function specView(spec: FakeSpec): SpecView_Serialize {
  switch (spec.mode) {
    case "inherit":
    case "none":
    case "prompt":
      return { mode: spec.mode };
    case "inline":
      return { mode: "inline", username: spec.username, domain: spec.domain };
    case "byId":
      return { mode: "by_id", target: spec.id };
    case "byName":
      return { mode: "by_name", name: spec.name };
  }
}

function nodeRef(doc: FakeDocument, id: string): NodeRef {
  return { id, path: pathOf(doc, id) };
}

/** The chain of a credential, as `resolve_trace`. */
export function traceOf(doc: FakeDocument, start: FakeNode): CredentialTrace_Serialize {
  if (start.kind === "credential") {
    return { steps: [], end: { kind: "credential", node: nodeRef(doc, start.id) } };
  }
  const steps: CredentialTrace_Serialize["steps"] = [];
  let current: FakeNode | null = start;
  while (current !== null) {
    steps.push({
      node: current.id,
      path: pathOf(doc, current.id),
      spec: specView(current.credential),
    });
    if (current.credential.mode !== "inherit") {
      return { steps, end: endOf(doc, current) };
    }
    current = findNode(doc, current.id).parent;
  }
  return { steps, end: { kind: "root" } };
}

function endOf(doc: FakeDocument, holder: FakeNode): TraceEnd {
  const spec = holder.credential;
  switch (spec.mode) {
    case "inherit":
      return { kind: "root" };
    case "none":
      return { kind: "none" };
    case "prompt":
      return { kind: "prompt" };
    case "inline":
      return { kind: "inline", node: nodeRef(doc, holder.id) };
    case "byId": {
      const target = [...walk(doc.nodes)].find(({ node }) => node.id === spec.id)?.node;
      if (!target) return { kind: "dangling", target: spec.id };
      return target.kind === "credential"
        ? { kind: "credential", node: nodeRef(doc, target.id) }
        : { kind: "not_a_credential", node: nodeRef(doc, target.id) };
    }
    case "byName": {
      const matches = credentialsNamed(doc, spec.name);
      if (matches.length === 0) return { kind: "name_not_found", name: spec.name };
      if (matches.length > 1) {
        return {
          kind: "name_ambiguous",
          name: spec.name,
          candidates: matches.map((node) => nodeRef(doc, node.id)),
        };
      }
      return { kind: "credential", node: nodeRef(doc, (matches[0] as FakeNode).id) };
    }
  }
}

/** The credentials named `name`: exactly, or else ignoring case (SPEC §4.3). */
export function credentialsNamed(doc: FakeDocument, name: string): FakeNode[] {
  const credentials = [...walk(doc.nodes)]
    .map(({ node }) => node)
    .filter((node) => node.kind === "credential");
  const exact = credentials.filter((node) => node.name === name);
  return exact.length > 0
    ? exact
    : credentials.filter((node) => node.name.toLowerCase() === name.toLowerCase());
}

/** The nodes whose spec names `target`, by id or by a name that resolves to it. */
export function referencesTo(doc: FakeDocument, target: string): FakeNode[] {
  return [...walk(doc.nodes)]
    .map(({ node }) => node)
    .filter((node) => {
      const spec = node.credential;
      if (node.kind !== "folder" && node.kind !== "connection") return false;
      if (spec.mode === "byId") return spec.id === target;
      if (spec.mode === "byName") {
        const matches = credentialsNamed(doc, spec.name);
        return matches.length === 1 && matches[0]?.id === target;
      }
      return jumpHostsOf(node).includes(target);
    });
}

function jumpHostsOf(node: FakeNode): string[] {
  const jumps = node.settings["jump_hosts"];
  return Array.isArray(jumps)
    ? jumps.flatMap((jump: unknown) =>
        typeof jump === "object" && jump !== null && "$ref" in jump ? [String(jump.$ref)] : [],
      )
    : [];
}

/** Who uses node `id`, as `credential_usage`. */
export function usageOf(doc: FakeDocument, node: FakeNode): CredentialUsage {
  const direct = referencesTo(doc, node.id);
  const inherited =
    node.kind === "credential"
      ? [...walk(doc.nodes)]
          .map(({ node: user }) => user)
          .filter(
            (user) =>
              user.kind === "connection" &&
              user.credential.mode === "inherit" &&
              !direct.includes(user) &&
              (() => {
                const end = traceOf(doc, user).end;
                return end.kind === "credential" && end.node.id === node.id;
              })(),
          )
      : [];
  return {
    direct: direct.map((user) => nodeRef(doc, user.id)),
    inherited: inherited.map((user) => nodeRef(doc, user.id)),
  };
}

export function slotOf(key: SlotKey): SecretSlot {
  if (key.startsWith("property:")) return { property: key.slice("property:".length) };
  return key === "privateKey" ? "private_key" : (key as "password" | "passphrase");
}

function secretViews(state: FakeBackendState, node: FakeNode): SecretView[] {
  return Object.entries(node.secrets).map(([key, present]) => ({
    slot: slotOf(key),
    state: state.vault.locked ? "vaultLocked" : present ? "present" : "absent",
  }));
}

export function detailsView(
  state: FakeBackendState,
  doc: FakeDocument,
  id: string,
): NodeDetailsView_Serialize {
  const { node, parent } = findNode(doc, id);
  const trace = node.kind === "unsupported" ? null : traceOf(doc, node);
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
            settings: structuredClone(node.settings) as NonNullable<
              NodeDetailsView_Serialize["connection"]
            >["settings"],
          }
        : null,
    credentialSpec:
      node.kind === "folder" || node.kind === "connection" ? specView(node.credential) : null,
    credential:
      node.kind === "credential"
        ? { username: node.username, domain: node.domain, keyFile: node.keyFile }
        : null,
    effective: trace === null ? null : effectiveOf(state, doc, trace.end),
    trace,
    secrets: secretViews(state, node),
    vaultLocked: state.vault.locked,
    usage: usageOf(doc, node),
    source: node.source ? { ...node.source, importedAt: node.created } : null,
    extra: structuredClone(node.extra) as NodeDetailsView_Serialize["extra"],
    unsupported: node.unsupported ? { ...node.unsupported } : null,
  };
}

function effectiveOf(
  state: FakeBackendState,
  doc: FakeDocument,
  end: TraceEnd,
): NodeDetailsView_Serialize["effective"] {
  const secretState = (present: boolean | undefined) =>
    present === undefined ? "not_set" : present || state.vault.locked ? "present" : "absent";
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
