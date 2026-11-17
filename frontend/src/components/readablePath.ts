import { texts } from "../i18n/it";

/**
 * Names from the root as the interface writes a path: "Clienti / ACME/Nord /
 * web01" (glossario, inventario §6). The one helper of the components and of
 * the features (DEC-039.4, DEC-045.3); the separator is the one of the texts.
 * The names stay as they are: the caller makes them visible where it shows
 * them.
 */
export function readableNames(names: readonly string[]): string {
  return names.join(texts.validation.separator);
}
