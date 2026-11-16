import type {
  CredentialChoice_Deserialize,
  IconChoice,
  JsonValue,
  NodeDetailsView_Serialize,
  SpecView_Serialize,
} from "../../bindings";
import { iconChoiceOf } from "../dialogs/iconChoice";

/** What the editor edits: the kinds of node it creates and changes. */
export type EditorKind = "ssh" | "rdp" | "folder" | "credential";

/** A custom property; a secret one shows its state, never its value. */
export interface PropertyRow {
  id: string;
  name: string;
  secret: boolean;
  /** The text of a text property. */
  text: string;
  /** The name a secret property has in the document, if it is saved. */
  saved: string | null;
}

/** A jump host: an SSH connection of the tree or a free address. */
export type JumpRow = { id: string; ref: string } | { id: string; address: string };

/** A free ssh option, key = value. */
export interface OptionRow {
  id: string;
  key: string;
  value: string;
}

/** The kinds of credential an RDP gateway offers in the editor (DEC-038.4). */
export type GatewayCredential =
  | Extract<CredentialChoice_Deserialize, { mode: "inherit" | "none" | "byId" | "byName" }>
  /** A spec the editor does not change (inline or asked, from an import): kept as stored. */
  | { mode: "kept"; stored: "inline" | "prompt"; username: string | null };

/** How a credential node holds its private key. */
export type KeyMode = "none" | "file" | "vault";

/** A change to a secret, applied by "Salva": the value itself is never here. */
export type SecretMark = "set" | "clear";

/** The fields of the form; those of the other kinds stay at their defaults. */
export interface EditorValues {
  kind: EditorKind;
  name: string;
  parent: string | null;
  description: string;
  notes: string;
  icon: IconChoice;
  color: string | null;
  favorite: boolean;
  tags: string[];
  properties: PropertyRow[];
  host: string;
  port: string;
  credential: CredentialChoice_Deserialize;
  jumpHosts: JumpRow[];
  agentForwarding: boolean;
  compression: boolean;
  x11: boolean;
  x11Trusted: boolean;
  keepalive: string;
  options: OptionRow[];
  resolution: "fit" | "fullscreen" | "fixed";
  width: string;
  height: string;
  scale: "100" | "140" | "180";
  colorDepth: "" | "15" | "16" | "24" | "32";
  gatewayHost: string;
  gatewayPort: string;
  gatewayCredential: GatewayCredential;
  multimon: boolean;
  admin: boolean;
  clipboard: boolean;
  wolMac: string;
  client: "auto" | "freerdp" | "windows-app";
  username: string;
  domain: string;
  keyMode: KeyMode;
  keyFile: string;
  /** The staged changes of the secrets, by slot (`password`, `property:<row id>`, …). */
  secrets: Record<string, SecretMark>;
}

/** The fields compared to find what changed, in the order of the form. */
export const EDITABLE_FIELDS = [
  "name",
  "parent",
  "description",
  "notes",
  "icon",
  "color",
  "favorite",
  "tags",
  "properties",
  "host",
  "port",
  "credential",
  "jumpHosts",
  "agentForwarding",
  "compression",
  "x11",
  "x11Trusted",
  "keepalive",
  "options",
  "resolution",
  "width",
  "height",
  "scale",
  "colorDepth",
  "gatewayHost",
  "gatewayPort",
  "gatewayCredential",
  "multimon",
  "admin",
  "clipboard",
  "wolMac",
  "client",
  "username",
  "domain",
  "keyMode",
  "keyFile",
] as const satisfies readonly (keyof EditorValues)[];

export type EditableField = (typeof EDITABLE_FIELDS)[number];

/** A new node of `kind` in `parent`: nothing filled in (NOTE.md: no example values). */
export function blankValues(kind: EditorKind, parent: string | null): EditorValues {
  return {
    kind,
    name: "",
    parent,
    description: "",
    notes: "",
    icon: { kind: "none" },
    color: null,
    favorite: false,
    tags: [],
    properties: [],
    host: "",
    port: "",
    credential: { mode: "inherit" },
    jumpHosts: [],
    agentForwarding: false,
    compression: false,
    x11: false,
    x11Trusted: false,
    keepalive: "",
    options: [],
    resolution: "fit",
    width: "",
    height: "",
    scale: "100",
    colorDepth: "",
    gatewayHost: "",
    gatewayPort: "",
    gatewayCredential: { mode: "inherit" },
    multimon: false,
    admin: false,
    // Shared by default, as in `RdpSettings` (SPEC §7.3).
    clipboard: true,
    wolMac: "",
    client: "auto",
    username: "",
    domain: "",
    keyMode: "none",
    keyFile: "",
    secrets: {},
  };
}

type JsonObject = { [key: string]: JsonValue };

function objectOf(value: JsonValue | undefined): JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? value : {};
}

function textOf(value: JsonValue | undefined): string {
  return typeof value === "string" ? value : typeof value === "number" ? String(value) : "";
}

function oneOf<T extends string>(value: JsonValue | undefined, allowed: readonly T[], fallback: T) {
  return allowed.find((option) => option === textOf(value)) ?? fallback;
}

/** The kind of the editor for a node of the backend; `null` for an unsupported node. */
export function editorKindOf(details: NodeDetailsView_Serialize): EditorKind | null {
  switch (details.kind) {
    case "folder":
    case "credential":
      return details.kind;
    case "connection":
      return details.connection?.protocol === "rdp" ? "rdp" : "ssh";
    case "unsupported":
      return null;
  }
}

/** The choice of the editor for a stored spec. */
export function choiceOfSpec(spec: SpecView_Serialize | null): CredentialChoice_Deserialize {
  switch (spec?.mode) {
    case undefined:
    case "inherit":
      return { mode: "inherit" };
    case "none":
    case "prompt":
      return { mode: spec.mode };
    case "inline":
      return { mode: "inline", username: spec.username, domain: spec.domain };
    case "by_id":
      return { mode: "byId", id: spec.target };
    case "by_name":
      return { mode: "byName", name: spec.name };
  }
}

/** The gateway credential of the editor for the spec stored in the settings. */
function gatewayCredentialOf(stored: JsonValue | undefined): GatewayCredential {
  const spec = objectOf(stored);
  switch (spec["mode"]) {
    case "none":
      return { mode: "none" };
    case "by_id":
      return { mode: "byId", id: textOf(spec["$ref"]) };
    case "by_name":
      return { mode: "byName", name: textOf(spec["name"]) };
    case "inline":
    case "prompt":
      return { mode: "kept", stored: spec["mode"], username: textOf(spec["username"]) || null };
    default:
      return { mode: "inherit" };
  }
}

function connectionValues(details: NodeDetailsView_Serialize): Partial<EditorValues> {
  const connection = details.connection;
  if (!connection) return {};
  const settings = connection.settings;
  const resolution = objectOf(settings["resolution"]);
  const gateway = objectOf(settings["gateway"]);
  const jumps = Array.isArray(settings["jump_hosts"]) ? settings["jump_hosts"] : [];
  return {
    host: connection.host,
    port: connection.port === null ? "" : String(connection.port),
    credential: choiceOfSpec(details.credentialSpec),
    jumpHosts: jumps.map((jump, index): JumpRow => {
      const entry = objectOf(jump);
      const id = `jump:${index}`;
      return "$ref" in entry
        ? { id, ref: textOf(entry["$ref"]) }
        : { id, address: textOf(entry["address"]) };
    }),
    agentForwarding: settings["agent_forwarding"] === true,
    compression: settings["compression"] === true,
    x11: settings["x11"] === true,
    x11Trusted: settings["x11_trusted"] === true,
    keepalive: textOf(settings["keepalive_s"]),
    options: Object.entries(objectOf(settings["options"])).map(([key, value]) => ({
      id: `option:${key}`,
      key,
      value: textOf(value),
    })),
    resolution: oneOf(resolution["mode"], ["fit", "fullscreen", "fixed"], "fit"),
    width: textOf(resolution["width"]),
    height: textOf(resolution["height"]),
    scale: oneOf(settings["scale"], ["100", "140", "180"], "100"),
    colorDepth: oneOf(settings["color_depth"], ["15", "16", "24", "32"], ""),
    gatewayHost: textOf(gateway["host"]),
    gatewayPort: textOf(gateway["port"]),
    gatewayCredential: gatewayCredentialOf(gateway["credential"]),
    multimon: settings["multimon"] === true,
    admin: settings["admin"] === true,
    clipboard: settings["clipboard"] !== false,
    wolMac: textOf(settings["wol_mac"]),
    client: oneOf(settings["client"], ["freerdp", "windows-app"], "auto"),
  };
}

function credentialValues(details: NodeDetailsView_Serialize): Partial<EditorValues> {
  const fields = details.credential;
  if (!fields) return {};
  const vaultKey = details.secrets.some((secret) => secret.slot === "private_key");
  return {
    username: fields.username ?? "",
    domain: fields.domain ?? "",
    keyFile: fields.keyFile ?? "",
    keyMode: fields.keyFile ? "file" : vaultKey ? "vault" : "none",
  };
}

/** The form of an existing node, as its detail describes it. */
export function valuesOf(kind: EditorKind, details: NodeDetailsView_Serialize): EditorValues {
  return {
    ...blankValues(kind, details.parent),
    name: details.name,
    description: details.description ?? "",
    notes: details.notes ?? "",
    icon: iconChoiceOf(details.icon),
    color: details.color,
    favorite: details.favorite,
    tags: [...details.tags],
    properties: details.properties.map((property) => ({
      // Ids from the content: the same detail gives the same rows (`externalChanges`).
      id: `property:${property.name}`,
      name: property.name,
      secret: property.secret,
      text: property.text ?? "",
      saved: property.secret ? property.name : null,
    })),
    ...(details.kind === "folder" ? { credential: choiceOfSpec(details.credentialSpec) } : {}),
    ...connectionValues(details),
    ...credentialValues(details),
  };
}
