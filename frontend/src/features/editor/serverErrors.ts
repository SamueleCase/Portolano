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
  /** The free ssh option refused, for the row of the options that has it. */
  option?: string;
}

/**
 * The settings of a connection as the backend names them in the field of
 * the error (`settings.<key>`, DEC-040.3). The settings chosen from a fixed
 * list (scale, color depth, client) have no field of their own.
 */
const SETTINGS_TARGETS: Readonly<Record<string, ErrorTarget>> = {
  options: "options",
  jump_hosts: "jumpHosts",
  keepalive_s: "keepalive",
  gateway: "gatewayHost",
  resolution: "width",
  wol_mac: "wolMac",
};

const SETTINGS = "settings.";
const OPTIONS = "settings.options.";
/** A secret property, by name (`property:<name>`). */
const PROPERTY = "property:";

/** `settings.<key>`, or `settings.options.<option>` for a refused ssh option. */
function settingsError(field: string, message: string): RoutedError {
  if (field.startsWith(OPTIONS)) {
    return { target: "options", message, option: field.slice(OPTIONS.length) };
  }
  return { target: SETTINGS_TARGETS[field.slice(SETTINGS.length)] ?? "form", message };
}

function fieldTarget(field: string, kind: EditorKind): ErrorTarget {
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
      // `settings` alone: the settings as a whole.
      return "form";
  }
}

export function routeError(error: PresentedError, kind: EditorKind): RoutedError {
  const field = error.presentation === "field" ? error.field : undefined;
  if (field === undefined) return { target: "form", message: error.message };
  if (field.startsWith(SETTINGS)) return settingsError(field, error.message);
  if (field.startsWith(PROPERTY)) return { target: "properties", message: error.message };
  return { target: fieldTarget(field, kind), message: error.message };
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
