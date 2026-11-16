/**
 * The paths of the backend (`path_of` of the core: `/Clienti/ACME/web01`,
 * with `\/` and `\\` inside a name) shown as the interface writes them:
 * "Clienti / ACME / web01" (inventario §6).
 */

/** The names of the nodes along `path`, unescaped. */
export function pathNames(path: string): string[] {
  const names: string[] = [];
  let current = "";
  let escaped = false;
  for (const character of path.startsWith("/") ? path.slice(1) : path) {
    if (escaped) {
      current += character;
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (character === "/") {
      names.push(current);
      current = "";
    } else {
      current += character;
    }
  }
  names.push(current);
  return names;
}

/** `path` with " / " between the names. */
export function readablePath(path: string): string {
  return pathNames(path).join(" / ");
}

/** The name of the last node of `path`. */
export function lastName(path: string): string {
  return pathNames(path).at(-1) ?? path;
}
