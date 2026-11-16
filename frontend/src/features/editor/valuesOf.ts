import type {
  CredentialChoice_Deserialize,
  JsonValue,
  NodeDetailsView_Serialize,
  SpecView_Serialize,
} from "../../bindings";
import { iconChoiceOf } from "../dialogs/iconChoice";
import {
  blankValues,
  type EditorKind,
  type EditorValues,
  type GatewayCredential,
  type JumpRow,
} from "./editorValues";

/** The form of the editor from the detail of an existing node. */

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
