import type { JsonValue } from "../bindings";
import type { FakeSpec, JsonObject } from "./fakeBackendModel";

/**
 * References inside the settings of a protocol, as `spec_scan` of the core
 * finds them without knowing the protocols: any object with `"$ref"` points
 * to a node (a jump host, a credential by id), and an object tagged with
 * `"mode"` in the serialized form of a credential spec is a credential (the
 * credential of an RDP gateway). Specs here use the names of the document
 * (`by_id`, `by_name`), not those of the fake (`byId`, `byName`).
 */

/** The steps from the settings to a value inside them. */
export type JsonPath = (string | number)[];

/** An object of the settings with its path. */
export interface EmbeddedObject {
  path: JsonPath;
  object: { [key: string]: JsonValue };
}

function isObject(value: JsonValue | undefined): value is { [key: string]: JsonValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Every object inside `settings`, in pre-order. */
export function embeddedObjects(settings: JsonObject): EmbeddedObject[] {
  const found: EmbeddedObject[] = [];
  const visit = (value: JsonValue, path: JsonPath) => {
    if (Array.isArray(value)) {
      value.forEach((item, index) => visit(item, [...path, index]));
    } else if (isObject(value)) {
      found.push({ path, object: value });
      for (const [key, inner] of Object.entries(value)) visit(inner, [...path, key]);
    }
  };
  for (const [key, value] of Object.entries(settings)) visit(value, [key]);
  return found;
}

/** The credential spec an object of the settings represents, if it has that form. */
export function embeddedSpec(object: { [key: string]: JsonValue }): FakeSpec | null {
  const ref = object["$ref"];
  const name = object["name"];
  const text = (key: string) => {
    const value = object[key];
    return typeof value === "string" ? value : null;
  };
  switch (object["mode"]) {
    case "inherit":
    case "none":
    case "prompt":
      return { mode: object["mode"] };
    case "inline":
      return { mode: "inline", username: text("username"), domain: text("domain") };
    case "by_id":
      return typeof ref === "string" ? { mode: "byId", id: ref } : null;
    case "by_name":
      return typeof name === "string" ? { mode: "byName", name } : null;
    default:
      return null;
  }
}

/** The credential specs inside `settings`, with their paths. */
export function embeddedSpecs(settings: JsonObject): { path: JsonPath; spec: FakeSpec }[] {
  return embeddedObjects(settings).flatMap(({ path, object }) => {
    const spec = embeddedSpec(object);
    return spec ? [{ path, spec }] : [];
  });
}

/** The nodes `settings` point to with `"$ref"`. */
export function embeddedRefs(settings: JsonObject): string[] {
  return embeddedObjects(settings).flatMap(({ object }) => {
    const ref = object["$ref"];
    return typeof ref === "string" ? [ref] : [];
  });
}

/** The value at `path` in `settings`, if there is one. */
export function valueAt(settings: JsonObject, path: JsonPath): JsonValue | undefined {
  let value: JsonValue | undefined = settings;
  for (const step of path) {
    if (Array.isArray(value) && typeof step === "number") value = value[step];
    else if (isObject(value) && typeof step === "string") value = value[step];
    else return undefined;
  }
  return value;
}

/** Puts `value` at `path` in `settings`, where its parent exists. */
export function setValueAt(settings: JsonObject, path: JsonPath, value: JsonValue): void {
  const parent = valueAt(settings, path.slice(0, -1));
  const last = path[path.length - 1];
  if (Array.isArray(parent) && typeof last === "number") parent[last] = value;
  else if (isObject(parent) && typeof last === "string") parent[last] = value;
}

/** The serialized form of a spec by id, as the document stores it. */
export function byIdValue(id: string): JsonValue {
  return { mode: "by_id", $ref: id };
}

/** The serialized form of "ask at launch". */
export const PROMPT_VALUE: JsonValue = { mode: "prompt" };
