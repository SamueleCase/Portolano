import type {
  ConfigEntry,
  ErrorCode,
  IconView,
  InterfacePreferences_Serialize,
  IpcErrorDetails,
  VaultKind,
} from "../bindings";
import type { IpcError } from "../ipc/result";

/**
 * The in-memory document of the fake backend. It mirrors the model of the
 * core closely enough for the screens, with one difference that matters:
 * secrets are never values, only "present" or "absent" in the fake vault,
 * so that no test can make the fake hold a password (SPEC-UI §1.1).
 * Invented data only: hosts under `example.com` and RFC 5737 (SPEC §1.3).
 */

/** The only master password the fake vault accepts (SPEC §1.3). */
export const TEST_MASTER_PASSWORD = "test-only-master-password";

/** A credential spec of a folder or a connection. */
export type FakeSpec =
  | { mode: "inherit" }
  | { mode: "none" }
  | { mode: "prompt" }
  | { mode: "inline"; username: string | null; domain: string | null }
  | { mode: "byId"; id: string }
  | { mode: "byName"; name: string };

/** Where a node holds a secret: `password`, `passphrase`, `privateKey`, `property:<name>`. */
export type SlotKey = string;

/** The icon stored in a node. */
export type FakeIcon =
  | { kind: "builtin"; name: string }
  | { kind: "royal"; name: string }
  | { kind: "custom"; sha256: string; mime: string };

export type FakeKind = "folder" | "connection" | "credential" | "unsupported";

export interface FakeNode {
  id: string;
  name: string;
  kind: FakeKind;
  description: string | null;
  notes: string | null;
  icon: FakeIcon | null;
  color: string | null;
  favorite: boolean;
  tags: string[];
  /** Custom properties; a secret one has no text. */
  properties: { name: string; text: string | null }[];
  created: string;
  modified: string;
  /** Connections. */
  protocol: string | null;
  host: string | null;
  port: number | null;
  settings: Record<string, unknown>;
  /** Folders and connections. */
  credential: FakeSpec;
  /** Credentials. */
  username: string | null;
  domain: string | null;
  keyFile: string | null;
  /** The secrets set on the node, with whether the fake vault has them. */
  secrets: Partial<Record<SlotKey, boolean>>;
  source: {
    system: string;
    sourceType: string;
    sourceId: string | null;
    file: string | null;
  } | null;
  extra: Record<string, unknown>;
  unsupported: { sourceType: string; raw: string } | null;
  children: FakeNode[];
}

export interface FakeDocument {
  name: string;
  path: string;
  nodes: FakeNode[];
}

/** A deletion that can still be undone. */
export interface FakePendingDelete {
  node: FakeNode;
  parent: string | null;
  index: number;
  /** The credentials that the deletion turned into "ask at launch". */
  detached: { id: string; credential: FakeSpec }[];
}

/** Everything the fake backend knows; tests change it freely before installing it. */
export interface FakeBackendState {
  defaultDocumentPath: string;
  vault: { kind: VaultKind; path: string | null; exists: boolean; locked: boolean };
  document: FakeDocument | null;
  /** Documents that "exist on disk", by path, to open them. */
  files: Record<string, FakeDocument>;
  recent: string[];
  preferences: InterfacePreferences_Serialize;
  config: ConfigEntry[];
  customIcons: IconView[];
  pendingDeletes: Record<string, FakePendingDelete>;
  /** What the native dialogs answer: a path, or null for "Annulla". */
  dialogs: {
    document: string | null;
    newDocument: string | null;
    importFile: string | null;
    icon: string | null;
    report: string | null;
  };
  /** Paths chosen in the dialogs, as the backend keeps them. */
  chosen: { document?: string; newDocument?: string; importFile?: string; icon?: string };
  /** Whether an import ran, for "Esporta JSON". */
  lastImport: boolean;
  nextId: number;
}

/** A failed command of the fake backend, thrown inside it and returned as the `IpcError`. */
export class FakeFailure extends Error {
  readonly error: IpcError;

  constructor(code: ErrorCode, message: string, details?: IpcErrorDetails) {
    super(message);
    this.name = "FakeFailure";
    this.error = details ? { code, message, details } : { code, message };
  }
}

/** Throws the error of `code`, as the Rust command would return it. */
export function fail(code: ErrorCode, message: string, details?: IpcErrorDetails): never {
  throw new FakeFailure(code, message, details);
}

/** Throws an invalid value shown under `field`. */
export function failField(field: string, message: string): never {
  fail("invalid", message, { kind: "invalid", field });
}

/** A fresh, unique id in UUID form. */
export function newId(state: FakeBackendState): string {
  state.nextId += 1;
  return `00000000-0000-4000-8000-${state.nextId.toString(16).padStart(12, "0")}`;
}

const NOW = "2026-11-10T10:00:00Z";

/** A node with every field at its default. */
export function blankNode(state: FakeBackendState, kind: FakeKind, name: string): FakeNode {
  return {
    id: newId(state),
    name,
    kind,
    description: null,
    notes: null,
    icon: null,
    color: null,
    favorite: false,
    tags: [],
    properties: [],
    created: NOW,
    modified: NOW,
    protocol: null,
    host: null,
    port: null,
    settings: {},
    credential: { mode: "inherit" },
    username: null,
    domain: null,
    keyFile: null,
    secrets: {},
    source: null,
    extra: {},
    unsupported: null,
    children: [],
  };
}

/** The open document, or `noDocument`. */
export function openDocument(state: FakeBackendState): FakeDocument {
  return state.document ?? fail("noDocument", "nessun documento aperto");
}

/** Every node of `nodes` and their descendants, in tree order, with their parent. */
export function* walk(
  nodes: FakeNode[],
  parent: FakeNode | null = null,
): Generator<{ node: FakeNode; parent: FakeNode | null }> {
  for (const node of nodes) {
    yield { node, parent };
    yield* walk(node.children, node);
  }
}

/** Node `id` with its parent, or `notFound` as `NodeNotFound` of the core. */
export function findNode(
  doc: FakeDocument,
  id: string,
): { node: FakeNode; parent: FakeNode | null } {
  for (const entry of walk(doc.nodes)) {
    if (entry.node.id === id) {
      return entry;
    }
  }
  return fail("notFound", `nodo ${id} non trovato`);
}

/** The children of `parent` (`null` is the root). */
export function childrenOf(doc: FakeDocument, parent: string | null): FakeNode[] {
  if (parent === null) {
    return doc.nodes;
  }
  const { node } = findNode(doc, parent);
  if (node.kind !== "folder") {
    fail("invalid", `il nodo ${parent} non è una cartella`);
  }
  return node.children;
}

/** The path of node `id`, as `path_of` of the core (with `/` and `\` escaped). */
export function pathOf(doc: FakeDocument, id: string): string {
  const chain: string[] = [];
  let current: string | null = id;
  while (current !== null) {
    const { node, parent }: { node: FakeNode; parent: FakeNode | null } = findNode(doc, current);
    chain.unshift(node.name.replaceAll("\\", "\\\\").replaceAll("/", "\\/"));
    current = parent?.id ?? null;
  }
  return `/${chain.join("/")}`;
}

/** The ids of `node` and of its descendants. */
export function subtreeIds(node: FakeNode): Set<string> {
  return new Set([...walk([node])].map(({ node: inner }) => inner.id));
}

/** Refuses an empty name or one a sibling other than `exclude` already has (DEC-022.5). */
export function checkName(siblings: FakeNode[], name: string, exclude?: string): void {
  if (name.trim() === "") {
    failField("name", "il nome del nodo non può essere vuoto");
  }
  if (siblings.some((sibling) => sibling.name === name && sibling.id !== exclude)) {
    failField("name", `esiste già un nodo chiamato '${name}'`);
  }
}

/** Refuses the commands that touch secrets while the file vault is locked. */
export function requireUnlockedVault(state: FakeBackendState): void {
  if (state.vault.locked) {
    fail("vaultLocked", "il vault è bloccato: sbloccalo con la master password");
  }
}
