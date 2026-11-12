import {
  BUILTIN_ICONS,
  type AppStatus_Serialize,
  type ChosenFile,
  type ConfigEntry,
  type DocumentInfo,
  type DocumentLocation,
  type DocumentSource,
  type IconView,
  type InterfacePreferences_Deserialize,
  type InterfacePreferences_Serialize,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  fail,
  failField,
  openDocument,
} from "./fakeBackendModel";

/** The document file, the configuration, the preferences and the icons. */

function fileName(path: string): string {
  return path.split("/").pop() ?? path;
}

function info(doc: FakeDocument): DocumentInfo {
  return { name: doc.name, fileName: fileName(doc.path), path: doc.path };
}

function location(state: FakeBackendState, path: string): DocumentLocation {
  return { path, exists: path in state.files };
}

export function appStatus(state: FakeBackendState): AppStatus_Serialize {
  const last = state.recent[0];
  return {
    defaultDocument: location(state, state.defaultDocumentPath),
    vault: state.vault.kind,
    document: state.document ? info(state.document) : null,
    lastDocument: last === undefined ? null : location(state, last),
    preferences: { ...state.preferences },
  };
}

/** A native dialog: its answer (set by the test) becomes the chosen file. */
export function choose(
  state: FakeBackendState,
  dialog: keyof FakeBackendState["dialogs"],
  chosen: keyof FakeBackendState["chosen"],
): ChosenFile | null {
  const path = state.dialogs[dialog];
  if (path === null) return null;
  state.chosen[chosen] = path;
  return { fileName: fileName(path) };
}

function sourcePath(
  state: FakeBackendState,
  source: DocumentSource,
  chosen: string | undefined,
): string {
  switch (source.kind) {
    case "chosen":
      return chosen ?? fail("notFound", "nessun file scelto");
    case "default":
      return state.defaultDocumentPath;
    case "recent":
      if (!state.recent.includes(source.path)) {
        fail("notFound", `'${source.path}' non è fra i documenti recenti`);
      }
      return source.path;
  }
}

function switchTo(state: FakeBackendState, doc: FakeDocument): DocumentInfo {
  state.pendingDeletes = {};
  state.document = doc;
  state.recent = [doc.path, ...state.recent.filter((path) => path !== doc.path)].slice(0, 10);
  return info(doc);
}

export function openDocumentAt(state: FakeBackendState, source: DocumentSource): DocumentInfo {
  const path = sourcePath(state, source, state.chosen.document);
  const doc = state.files[path] ?? fail("documentMissing", `il documento '${path}' non esiste`);
  return switchTo(state, doc);
}

export function createDocument(
  state: FakeBackendState,
  source: DocumentSource,
  name: string,
): DocumentInfo {
  const path = sourcePath(state, source, state.chosen.newDocument);
  if (path in state.files) fail("invalid", `il file '${path}' esiste già`);
  const doc: FakeDocument = { name, path, nodes: [] };
  state.files[path] = doc;
  return switchTo(state, doc);
}

export function recentDocuments(state: FakeBackendState): DocumentLocation[] {
  return state.recent.map((path) => location(state, path));
}

export function reloadDocument(state: FakeBackendState): DocumentInfo {
  return info(openDocument(state));
}

export function closeDocument(state: FakeBackendState): null {
  state.pendingDeletes = {};
  state.document = null;
  return null;
}

/** The keys whose value is a file, chosen with a dialog. */
const FILE_KEYS = ["default_document", "vault.file", "ssh.program", "rdp.freerdp_program"];

const CONFIG_VALUES: Record<string, string[]> = {
  "vault.backend": ["keychain", "file"],
  "terminal.app": ["terminal", "iterm", "ghostty", "custom"],
  "rdp.client": ["auto", "freerdp", "windows-app"],
  "rdp.cert": ["tofu", "ignore", "deny"],
};

const CONFIG_KEYS = [...Object.keys(CONFIG_VALUES), "terminal.command", ...FILE_KEYS];

function setEntry(state: FakeBackendState, key: string, value: string | null): ConfigEntry[] {
  if (!CONFIG_KEYS.includes(key)) {
    failField(key, `chiave di configurazione sconosciuta '${key}'`);
  }
  const allowed = CONFIG_VALUES[key];
  if (value !== null && allowed && !allowed.includes(value)) {
    failField(key, `valore non valido per '${key}': ${value}`);
  }
  state.config = state.config.filter((entry) => entry.key !== key);
  if (value !== null) state.config.push({ key, value });
  state.config.sort((a, b) => a.key.localeCompare(b.key));
  if (key === "vault.backend" && value !== null) {
    state.vault.kind = value === "file" ? "file" : "keychain";
  }
  return [...state.config];
}

export function configSet(state: FakeBackendState, key: string, value: string): ConfigEntry[] {
  if (FILE_KEYS.includes(key)) failField(key, "questo percorso si sceglie con «Scegli…»");
  return setEntry(state, key, value);
}

export function configReset(state: FakeBackendState, key: string): ConfigEntry[] {
  return setEntry(state, key, null);
}

export function chooseConfigFile(state: FakeBackendState, key: string): ConfigEntry[] | null {
  if (!FILE_KEYS.includes(key)) failField(key, `'${key}' non è un percorso di file`);
  const path = state.dialogs.document;
  return path === null ? null : setEntry(state, key, path);
}

export function uiPrefsSet(
  state: FakeBackendState,
  preferences: InterfacePreferences_Deserialize,
): InterfacePreferences_Serialize {
  state.preferences = { ...state.preferences, ...preferences };
  return { ...state.preferences };
}

export function builtinIcons(): string[] {
  return [...BUILTIN_ICONS];
}

export function customIcons(state: FakeBackendState): IconView[] {
  openDocument(state);
  return [...state.customIcons];
}

/** Stores the chosen image: a fake hash of its name, served as an image of the document. */
export function addIcon(state: FakeBackendState): IconView {
  openDocument(state);
  const path = state.chosen.icon ?? fail("notFound", "nessuna immagine scelta");
  const sha256 = [...path]
    .reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) % 2 ** 32, 7)
    .toString(16)
    .padStart(64, "0");
  const icon: IconView = {
    kind: "custom",
    sha256,
    mime: "image/png",
    url: `portolano-icon://localhost/${sha256}.png`,
  };
  if (
    !state.customIcons.some((existing) => existing.kind === "custom" && existing.sha256 === sha256)
  ) {
    state.customIcons.push(icon);
  }
  return icon;
}
