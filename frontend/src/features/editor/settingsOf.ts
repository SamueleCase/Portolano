import type { JsonValue } from "../../bindings";
import type { EditorValues, GatewayCredential } from "./editorValues";

/**
 * The `settings` of a connection from the form (`SshSettings` and
 * `RdpSettings` of the connectors): the keys the editor manages are written
 * as the backend stores them, with the defaults left out; every other key of
 * the stored settings stays as it was.
 */

export type JsonObject = { [key: string]: JsonValue };

const SSH_KEYS = [
  "jump_hosts",
  "agent_forwarding",
  "x11",
  "x11_trusted",
  "compression",
  "keepalive_s",
  "options",
];

const RDP_KEYS = [
  "resolution",
  "scale",
  "color_depth",
  "gateway",
  "multimon",
  "admin",
  "clipboard",
  "wol_mac",
  "client",
];

/** `value` if it is a JSON object, else an empty one. */
export function objectOf(value: JsonValue | undefined): JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? value : {};
}

/** The integer of a field already checked by the schema; `undefined` when empty. */
export function integerOf(text: string): number | undefined {
  return text.trim() === "" ? undefined : Number(text.trim());
}

function without(stored: JsonObject, keys: readonly string[]): JsonObject {
  return Object.fromEntries(Object.entries(stored).filter(([key]) => !keys.includes(key)));
}

/** Adds `key` only when `value` is not the default (`undefined` or `false`). */
function put(settings: JsonObject, key: string, value: JsonValue | undefined | false): void {
  if (value !== undefined && value !== false) settings[key] = value;
}

function sshSettings(values: EditorValues, stored: JsonObject): JsonObject {
  const settings = without(stored, SSH_KEYS);
  if (values.jumpHosts.length > 0) {
    settings["jump_hosts"] = values.jumpHosts.map((jump) =>
      "ref" in jump ? { $ref: jump.ref } : { address: jump.address.trim() },
    );
  }
  put(settings, "agent_forwarding", values.agentForwarding);
  put(settings, "x11", values.x11);
  put(settings, "x11_trusted", values.x11 && values.x11Trusted);
  put(settings, "compression", values.compression);
  put(settings, "keepalive_s", integerOf(values.keepalive));
  const options = values.options.filter((option) => option.key.trim() !== "");
  if (options.length > 0) {
    settings["options"] = Object.fromEntries(
      options.map((option) => [option.key.trim(), option.value]),
    );
  }
  return settings;
}

/** The credential of the gateway as the document stores it (a `CredentialSpec`). */
function gatewaySpec(credential: GatewayCredential, stored: JsonValue | undefined) {
  switch (credential.mode) {
    case "inherit":
      return undefined;
    case "none":
      return { mode: "none" };
    case "byId":
      return { mode: "by_id", $ref: credential.id };
    case "byName":
      return { mode: "by_name", name: credential.name };
    case "kept":
      return stored;
  }
}

function rdpSettings(values: EditorValues, stored: JsonObject): JsonObject {
  const settings = without(stored, RDP_KEYS);
  if (values.resolution === "fullscreen") settings["resolution"] = { mode: "fullscreen" };
  if (values.resolution === "fixed") {
    settings["resolution"] = {
      mode: "fixed",
      width: integerOf(values.width) ?? 0,
      height: integerOf(values.height) ?? 0,
    };
  }
  // 100% is the scale of FreeRDP without `/scale:`.
  put(settings, "scale", values.scale === "100" ? undefined : Number(values.scale));
  put(settings, "color_depth", integerOf(values.colorDepth));
  if (values.gatewayHost.trim() !== "") {
    const storedGateway = objectOf(stored["gateway"]);
    const gateway: JsonObject = { host: values.gatewayHost.trim() };
    put(gateway, "port", integerOf(values.gatewayPort));
    put(gateway, "credential", gatewaySpec(values.gatewayCredential, storedGateway["credential"]));
    settings["gateway"] = gateway;
  }
  put(settings, "multimon", values.multimon);
  put(settings, "admin", values.admin);
  // Shared by default: only "off" is stored.
  if (!values.clipboard) settings["clipboard"] = false;
  put(settings, "wol_mac", values.wolMac.trim() === "" ? undefined : values.wolMac.trim());
  put(settings, "client", values.client === "auto" ? undefined : values.client);
  return settings;
}

/** The settings of a connection of the form, over the stored ones. */
export function settingsOf(values: EditorValues, stored: JsonObject = {}): JsonObject {
  return values.kind === "rdp" ? rdpSettings(values, stored) : sshSettings(values, stored);
}
