import type { CredentialChoice_Deserialize, IconChoice } from "../../bindings";

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
