import type { DuplicateOutcome } from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeNode,
  findNode,
  newId,
  openDocument,
  pathOf,
  requireVaultFor,
  secretsIn,
  siblingsOf,
  walk,
} from "./fakeBackendModel";
import { credentialsNamed } from "./fakeBackendViews";

/**
 * "Duplica" as `duplicate_node` (DEC-016.4, DEC-031.3): new ids, references
 * inside the copy pointing to the copy (by id, and by name when the name
 * resolves to a copied credential), copied credentials that some reference
 * by name looks for renamed so that no name becomes ambiguous, no import
 * origin. Secrets need the vault, which copies them.
 */
export function duplicateNode(state: FakeBackendState, id: string): DuplicateOutcome {
  const doc = openDocument(state);
  const { node, parent } = findNode(doc, id);
  requireVaultFor(state, secretsIn(node) > 0);
  const copy = structuredClone(node);
  const copies = new Map<string, string>();
  for (const { node: inner } of walk([copy])) {
    const fresh = newId(state);
    copies.set(inner.id, fresh);
    inner.id = fresh;
    inner.source = null;
  }
  for (const { node: inner } of walk([copy])) retarget(doc, inner, copies);
  const renamed = renameWanted(doc, copy);
  const siblings = siblingsOf(doc, parent);
  if (!renamed.includes(copy.id)) {
    copy.name = freeName(node.name, (name) => siblings.some((sibling) => sibling.name === name));
  }
  siblings.splice(siblings.indexOf(node) + 1, 0, copy);
  const states = [...walk([copy])].flatMap(({ node: inner }) => Object.values(inner.secrets));
  return {
    id: copy.id,
    path: pathOf(doc, copy.id),
    renamed: renamed.map((renamedId) => ({ id: renamedId, path: pathOf(doc, renamedId) })),
    secrets_copied: states.filter(Boolean).length,
    secrets_missing: states.filter((present) => !present).length,
  };
}

/** `<name> (copia)`, then `(copia 2)`, … the first that `taken` refuses not. */
function freeName(name: string, taken: (candidate: string) => boolean): string {
  let candidate = `${name} (copia)`;
  for (let counter = 2; taken(candidate); counter += 1) candidate = `${name} (copia ${counter})`;
  return candidate;
}

/** Points the references of `node` to the copies of their targets. */
function retarget(doc: FakeDocument, node: FakeNode, copies: Map<string, string>): void {
  const spec = node.credential;
  if (spec.mode === "byId") {
    node.credential = { mode: "byId", id: copies.get(spec.id) ?? spec.id };
  } else if (spec.mode === "byName") {
    const matches = credentialsNamed(doc, spec.name);
    const copied = matches.length === 1 ? copies.get(matches[0]?.id ?? "") : undefined;
    if (copied !== undefined) node.credential = { mode: "byId", id: copied };
  }
  const jumps = node.settings["jump_hosts"];
  if (Array.isArray(jumps)) {
    node.settings["jump_hosts"] = jumps.map((jump) => {
      const target = typeof jump === "object" && jump !== null && "$ref" in jump ? jump.$ref : null;
      return typeof target === "string" ? { $ref: copies.get(target) ?? target } : jump;
    });
  }
}

/** Renames the copied credentials whose name a reference by name looks for. */
function renameWanted(doc: FakeDocument, copy: FakeNode): string[] {
  const wanted = new Set(
    [...walk(doc.nodes)]
      .map(({ node }) => node.credential)
      .flatMap((spec) => (spec.mode === "byName" ? [spec.name.toLowerCase()] : [])),
  );
  const taken = new Set(
    [...walk(doc.nodes), ...walk([copy])].map(({ node }) => node.name.toLowerCase()),
  );
  const renamed: string[] = [];
  for (const { node } of walk([copy])) {
    if (node.kind !== "credential" || !wanted.has(node.name.toLowerCase())) continue;
    node.name = freeName(node.name, (candidate) => taken.has(candidate.toLowerCase()));
    taken.add(node.name.toLowerCase());
    renamed.push(node.id);
  }
  return renamed;
}
