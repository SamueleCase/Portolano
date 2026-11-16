import type { DotState } from "../../components/StatusDot";
import type { TreeRowMeta } from "../../components/TreeRow";
import { visibleText } from "../../components/visibleText";
import { plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import type { TreeEntry } from "./treeModel";

/**
 * The accessible name of a row: everything the row shows, in words, since
 * the dot of the session and the colors are not read (SPEC-UI §1.6):
 * "web01, Connessione SSH, web01.example.com, preferito, connesso".
 */
export function rowLabel(
  entry: TreeEntry,
  meta: TreeRowMeta | undefined,
  session: DotState,
  expanded: boolean,
): string {
  const t = texts.components;
  const parts = [visibleText(entry.node.name), t.nodeKinds[entry.kind]];
  if (meta?.kind === "host") parts.push(visibleText(meta.host));
  if (meta?.kind === "descendants" && !expanded) parts.push(plural(meta.count, t.tree.descendants));
  if (meta?.kind === "secret") {
    parts.push(
      meta.state === "present" ? t.tree.meta.present[meta.backend] : t.tree.meta[meta.state],
    );
  }
  if (entry.node.favorite) parts.push(texts.tree.favorite);
  if (session !== "idle") parts.push(t.statusDot[session].toLowerCase());
  return parts.join(", ");
}
