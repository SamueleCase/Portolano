/**
 * The names of the builtin icons, as `Icon::Builtin { name }` stores them in
 * the document (DEC-029.4). The backend exports the same list; a test keeps
 * the two aligned once the bindings carry it.
 */
export const builtinIconNames = [
  "server",
  "desktop",
  "laptop",
  "database",
  "terminal",
  "cloud",
  "network",
  "router",
  "globe",
  "key",
  "lock",
  "shield",
  "folder",
  "hard-drive",
  "cpu",
  "container",
  "box",
  "building",
  "home",
  "user",
  "users",
  "mail",
  "printer",
  "phone",
  "wifi",
  "git-branch",
  "code",
  "settings",
  "star",
  "tag",
] as const;

/** The name of a builtin icon. */
export type BuiltinIconName = (typeof builtinIconNames)[number];

/** Whether `name`, read from the document, is a builtin icon. */
export function isBuiltinIconName(name: string): name is BuiltinIconName {
  return builtinIconNames.some((builtin) => builtin === name);
}
