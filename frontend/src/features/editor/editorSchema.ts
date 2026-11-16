import { z } from "zod";

import { texts } from "../../i18n/it";
import type { EditorValues } from "./editorValues";
import { PRIVATE_KEY } from "./secretSlots";

/**
 * The checks of form only (SPEC-UI §5.4): what can be said without the
 * document, such as a port out of range or an empty name. The rules of the
 * domain (names already taken, ssh options allowed, …) are the backend's,
 * and its errors go under the field it names.
 */

const PORT_MAX = 65535;
const INTEGER = /^\d+$/;

function isPort(text: string): boolean {
  const value = text.trim();
  return value === "" || (INTEGER.test(value) && Number(value) >= 1 && Number(value) <= PORT_MAX);
}

function isCount(text: string, min: number): boolean {
  return INTEGER.test(text.trim()) && Number(text.trim()) >= min;
}

export const editorSchema = z.custom<EditorValues>().superRefine((values, context) => {
  const t = texts.editor.errors;
  const issue = (field: keyof EditorValues, message: string) =>
    context.addIssue({ code: "custom", path: [field], message });
  const connection = values.kind === "ssh" || values.kind === "rdp";

  if (values.name.trim() === "") issue("name", t.name);
  const names = values.properties.map((row) => row.name.trim());
  if (names.some((name, index) => name === "" || names.indexOf(name) !== index)) {
    issue("properties", t.propertyName);
  }
  if (values.properties.some((row) => row.saved !== null && row.name.trim() !== row.saved)) {
    issue("properties", t.renamedSecret);
  }
  if (connection && values.host.trim() === "") issue("host", t.host);
  if (connection && !isPort(values.port)) issue("port", t.port);
  if (values.kind === "ssh") {
    if (values.keepalive.trim() !== "" && !isCount(values.keepalive, 0)) {
      issue("keepalive", t.keepalive);
    }
    if (values.options.some((option) => option.key.trim() === "" && option.value !== "")) {
      issue("options", t.optionKey);
    }
  }
  if (values.kind === "rdp") {
    if (values.resolution === "fixed" && !(isCount(values.width, 1) && isCount(values.height, 1))) {
      issue("width", t.size);
    }
    if (values.gatewayHost.trim() !== "" && !isPort(values.gatewayPort)) {
      issue("gatewayPort", t.port);
    }
  }
  if (values.kind === "credential") {
    if (values.keyMode === "file" && values.keyFile.trim() === "") issue("keyFile", t.keyFile);
  }
});

/** Whether a vault key is missing: "Nel vault" without a key saved or staged. */
export function missingVaultKey(values: EditorValues, storedKey: boolean): boolean {
  return (
    values.kind === "credential" &&
    values.keyMode === "vault" &&
    values.secrets[PRIVATE_KEY] !== "set" &&
    (!storedKey || values.secrets[PRIVATE_KEY] === "clear")
  );
}
