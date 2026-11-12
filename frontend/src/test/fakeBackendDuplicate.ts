import type { DuplicateOutcome } from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeNode,
  type FakeSpec,
  findNode,
  newId,
  openDocument,
  pathOf,
  requireVaultFor,
  secretsIn,
  siblingsOf,
  walk,
} from "./fakeBackendModel";
import { embeddedObjects, embeddedSpec, embeddedSpecs } from "./fakeBackendSettings";
import { specTarget } from "./fakeBackendViews";

/**
 * "Duplica" as `duplicate_node` (DEC-016.4, DEC-031.3): new ids, references
 * inside the copy pointing to the copy (by id, and by name when the name
 * resolves to a copied credential, on the node and in its settings), every
 * copied credential renamed to a name no node has and no reference by name
 * looks for, no import origin. Secrets need the vault, which copies them.
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
  const renamed = renameCredentials(doc, copy);
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

/** Points the references of `node` to the copies of their targets, on the node and in its settings. */
function retarget(doc: FakeDocument, node: FakeNode, copies: Map<string, string>): void {
  const copyOf = (spec: FakeSpec) => {
    const target = specTarget(doc, spec);
    return target === null ? undefined : copies.get(target);
  };
  const ownCopy = copyOf(node.credential);
  if (ownCopy !== undefined) node.credential = { mode: "byId", id: ownCopy };
  for (const { object } of embeddedObjects(node.settings)) {
    const spec = embeddedSpec(object);
    const ref = object["$ref"];
    if (spec?.mode === "byName") {
      const copied = copyOf(spec);
      if (copied === undefined) continue;
      delete object["name"];
      object["mode"] = "by_id";
      object["$ref"] = copied;
    } else if (typeof ref === "string") {
      object["$ref"] = copies.get(ref) ?? ref;
    }
  }
}

/** The names, lowercase, that references by name of `doc` look for, resolved or not. */
function wantedNames(doc: FakeDocument): Set<string> {
  return new Set(
    [...walk(doc.nodes)]
      .flatMap(({ node }) => [
        node.credential,
        ...embeddedSpecs(node.settings).map(({ spec }) => spec),
      ])
      .flatMap((spec) => (spec.mode === "byName" ? [spec.name.toLowerCase()] : [])),
  );
}

/**
 * Gives every copied credential a name that no node has and no reference
 * by name looks for, ignoring case, as `rename_credentials` (DEC-031.3).
 */
function renameCredentials(doc: FakeDocument, copy: FakeNode): string[] {
  const taken = new Set([
    ...[...walk(doc.nodes), ...walk([copy])].map(({ node }) => node.name.toLowerCase()),
    ...wantedNames(doc),
  ]);
  const renamed: string[] = [];
  for (const { node } of walk([copy])) {
    if (node.kind !== "credential") continue;
    node.name = freeName(node.name, (candidate) => taken.has(candidate.toLowerCase()));
    taken.add(node.name.toLowerCase());
    renamed.push(node.id);
  }
  return renamed;
}
