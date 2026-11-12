import {
  BUILTIN_ICONS,
  type CredentialTrace_Serialize,
  type CredentialUsage,
  type IconView,
  type NodeKindView,
  type NodeRef,
  type SecretsBadge,
  type SpecView_Serialize,
  type TraceEnd,
  type TreeNode_Serialize,
  type TreeView_Serialize,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeIcon,
  type FakeNode,
  type FakeSpec,
  findNode,
  isLocked,
  pathOf,
  walk,
} from "./fakeBackendModel";
import { embeddedRefs, embeddedSpecs } from "./fakeBackendSettings";

/** The read-only views of the fake document: what `tree` and `node_details` return. */

export const DEFAULT_PORTS: Partial<Record<string, number>> = { ssh: 22, rdp: 3389 };

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

/** The extension of the stored images of each type, as the icon store names them. */
const EXTENSIONS: Partial<Record<string, string>> = {
  "image/png": "png",
  "image/jpeg": "jpg",
  "image/gif": "gif",
  "image/tiff": "tiff",
  "image/svg+xml": "svg",
  "application/pdf": "pdf",
};

/** The icon to show, as `IconView::of`: Royal names through the words of the backend. */
export function iconView(icon: FakeIcon | null): IconView | null {
  if (icon === null) return null;
  switch (icon.kind) {
    case "builtin":
      // A name this version does not know shows the icon of the kind.
      return BUILTIN_ICONS.some((name) => name === icon.name)
        ? { kind: "builtin", name: icon.name }
        : null;
    case "royal":
      // Only the word `server` of the table of the backend is simulated.
      return icon.name.split(/[^A-Za-z0-9]+/).some((word) => word.toLowerCase() === "server")
        ? { kind: "builtin", name: "server" }
        : null;
    case "custom": {
      const extension = EXTENSIONS[icon.mime];
      if (extension === undefined) return null;
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
  if (isLocked(state)) return "vaultLocked";
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

export function specView(spec: FakeSpec): SpecView_Serialize {
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
      const [only] = matches;
      return only
        ? { kind: "credential", node: nodeRef(doc, only.id) }
        : { kind: "name_not_found", name: spec.name };
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

/** The credential a spec points to: by id, or by a name that resolves to one credential only. */
export function specTarget(doc: FakeDocument, spec: FakeSpec): string | null {
  if (spec.mode === "byId") return spec.id;
  if (spec.mode === "byName") {
    const matches = credentialsNamed(doc, spec.name);
    return matches.length === 1 ? (matches[0]?.id ?? null) : null;
  }
  return null;
}

/** The nodes that refer to `target`: by their credential or as a jump host (`references_to`). */
export function referencesTo(doc: FakeDocument, target: string): FakeNode[] {
  return [...walk(doc.nodes)]
    .map(({ node }) => node)
    .filter(
      (node) =>
        ((node.kind === "folder" || node.kind === "connection") &&
          specTarget(doc, node.credential) === target) ||
        embeddedRefs(node.settings).includes(target) ||
        embeddedSpecs(node.settings).some(({ spec }) => specTarget(doc, spec) === target),
    );
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
