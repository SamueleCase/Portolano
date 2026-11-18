import { readableNames } from "../../components/readablePath";

/**
 * A path of the import report or of a password request, "Ufficio/Rete/web01"
 * with the escapes of SPEC §4.2 (`\/` and `\\` inside a name), as the
 * glossary shows paths: "Ufficio / Rete / web01" (DEC-039.4). Only the
 * presentation changes: the names are those of the importer.
 */
export function readableImportPath(path: string): string {
  const names: string[] = [];
  let name = "";
  for (let index = 0; index < path.length; index += 1) {
    const character = path[index];
    if (character === "\\" && index + 1 < path.length) {
      index += 1;
      name += path[index];
    } else if (character === "/") {
      names.push(name);
      name = "";
    } else {
      name += character;
    }
  }
  names.push(name);
  return readableNames(names);
}
