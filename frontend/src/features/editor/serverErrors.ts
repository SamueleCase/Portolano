import type { PresentedError } from "../../ipc/errors";
import type { EditableField, EditorKind } from "./editorValues";

/**
 * Where an error of the backend goes in the editor: under the field that
 * `details.field` names (SPEC-UI §4.2), else at the bottom of the panel.
 */
export type ErrorTarget = EditableField | "password" | "passphrase" | "privateKey" | "form";

export interface RoutedError {
  target: ErrorTarget;
  message: string;
}

/** The fields of the settings of a connection, as the messages of the connectors name them. */
const SETTINGS_TARGETS: Readonly<Record<string, ErrorTarget>> = {
  options: "options",
  jump_hosts: "jumpHosts",
  keepalive_s: "keepalive",
  gateway: "gatewayHost",
  resolution: "width",
  wol_mac: "wolMac",
};

/**
 * The backend names only `settings` for a refused setting; its message
 * starts with the setting, as `campo 'options': …`. The settings chosen
 * from a fixed list (scale, color depth, client) have no field of their own.
 */
function settingsTarget(message: string): ErrorTarget {
  const setting = /campo '([a-z_]+)'/.exec(message)?.[1];
  return (setting === undefined ? undefined : SETTINGS_TARGETS[setting]) ?? "form";
}

function fieldTarget(field: string, message: string, kind: EditorKind): ErrorTarget {
  switch (field) {
    case "name":
    case "color":
    case "icon":
    case "host":
    case "port":
    case "credential":
      return field;
    case "domain":
      return kind === "credential" ? "domain" : "credential";
    case "protocol":
      return "host";
    case "settings":
      return settingsTarget(message);
    case "user":
    case "username":
      return kind === "credential" ? "username" : "credential";
    case "key_file":
      return "keyFile";
    case "private_key":
      return kind === "credential" ? "privateKey" : "form";
    case "password":
      return kind === "credential" ? "password" : "credential";
    case "passphrase":
      return "passphrase";
    default:
      return "form";
  }
}

export function routeError(error: PresentedError, kind: EditorKind): RoutedError {
  const target =
    error.presentation === "field" && error.field
      ? fieldTarget(error.field, error.message, kind)
      : "form";
  return { target, message: error.message };
}

const GENERAL: readonly ErrorTarget[] = [
  "name",
  "parent",
  "description",
  "notes",
  "icon",
  "color",
  "favorite",
  "tags",
  "properties",
];

/** The step that shows `target`: 0 Generale, 1 the step of the kind. */
export function stepOf(target: ErrorTarget): 0 | 1 {
  return GENERAL.includes(target) ? 0 : 1;
}

/** The option of the free ssh options a message names (`l'opzione 'ProxyCommand'`). */
export function optionNamed(message: string, keys: readonly string[]): string | undefined {
  return keys.find((key) => key !== "" && message.includes(`'${key}'`));
}
