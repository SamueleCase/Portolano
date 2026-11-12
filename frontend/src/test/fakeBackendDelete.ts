import type {
  DeleteOptionsInput_Deserialize,
  DeletePreview,
  DeleteResult,
  RestoreOutcome,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeDocument,
  type FakeNode,
  type FakeDetached,
  type FakeSpec,
  UNDO_WINDOW_MS,
  checkName,
  childrenOf,
  fail,
  findNode,
  newId,
  openDocument,
  pathOf,
  requireVaultFor,
  secretsIn,
  siblingsOf,
  subtreeIds,
  walk,
} from "./fakeBackendModel";
import { PROMPT_VALUE, embeddedSpecs, setValueAt, valueAt } from "./fakeBackendSettings";
import { referencesTo, specTarget, usageOf } from "./fakeBackendViews";

/** Deletion with undo, as `delete_undoable` and `restore_deleted`: the subtree goes, its secrets wait. */

function externalReferences(state: FakeBackendState, node: FakeNode) {
  const doc = openDocument(state);
  const subtree = subtreeIds(node);
  return [...walk([node])].flatMap(({ node: target }) =>
    referencesTo(doc, target.id)
      .filter((from) => !subtree.has(from.id))
      .map((from) => ({
        from: from.id,
        from_path: pathOf(doc, from.id),
        target: target.id,
        target_path: pathOf(doc, target.id),
      })),
  );
}

export function previewDelete(state: FakeBackendState, id: string): DeletePreview {
  const doc = openDocument(state);
  const { node } = findNode(doc, id);
  const subtree = subtreeIds(node);
  const inherited = [...walk([node])]
    .filter(({ node: inner }) => inner.kind === "credential")
    .flatMap(({ node: credential }) => usageOf(doc, credential).inherited)
    .filter((user) => !subtree.has(user.id));
  return {
    id,
    path: pathOf(doc, id),
    node_type:
      node.kind === "connection"
        ? { type: "connection", protocol: node.protocol ?? "" }
        : node.kind === "unsupported"
          ? { type: "unsupported", source_type: node.unsupported?.sourceType ?? "" }
          : { type: node.kind },
    children: node.children.length,
    descendants: subtree.size - 1,
    secrets: secretsIn(node),
    references: externalReferences(state, node),
    inherited,
  };
}

export function deleteNode(
  state: FakeBackendState,
  id: string,
  options: DeleteOptionsInput_Deserialize,
): DeleteResult {
  const doc = openDocument(state);
  const { node, parent } = findNode(doc, id);
  if (node.children.length > 0 && !options.recursive) {
    fail(
      "invalid",
      `la cartella '${pathOf(doc, id)}' contiene ${node.children.length} elementi: serve l'eliminazione ricorsiva`,
    );
  }
  const references = externalReferences(state, node);
  if (references.length > 0 && !options.force) {
    fail(
      "referenced",
      `'${pathOf(doc, id)}' è usato da ${references.length} riferimenti esterni: eliminazione rifiutata`,
      { kind: "referenced", references },
    );
  }
  // The secrets stay in the vault until the deletion expires, but the vault
  // is checked now, as `delete_undoable` does.
  requireVaultFor(state, secretsIn(node) > 0);
  const detached = options.force ? detachCredentials(doc, node) : [];
  const siblings = siblingsOf(doc, parent);
  const index = siblings.indexOf(node);
  siblings.splice(index, 1);
  const remaining = references.filter((reference) =>
    referencesTo(doc, reference.target).some((from) => from.id === reference.from),
  );
  const undoId = newId(state);
  state.pendingDeletes[undoId] = {
    node,
    parent: parent?.id ?? null,
    index,
    detached,
    expiresAt: state.clock() + UNDO_WINDOW_MS,
  };
  return {
    outcome: {
      removed_nodes: subtreeIds(node).size,
      detached: detached.map((entry) => entry.id),
      remaining_references: remaining,
      secrets_deleted: 0,
      cleanup_failures: [],
    },
    undoId,
  };
}

/**
 * Turns into "ask at launch" the credentials outside `node`'s subtree that
 * point to a credential of it, by id or by a name that resolves to it, on
 * the node or in its settings, as `detach_credential_references`; other
 * references, such as jump hosts, stay (dangling).
 */
function detachCredentials(doc: FakeDocument, node: FakeNode): FakeDetached[] {
  const subtree = subtreeIds(node);
  const credentials = new Set(
    [...walk([node])]
      .filter(({ node: inner }) => inner.kind === "credential")
      .map(({ node: inner }) => inner.id),
  );
  const pointsInside = (spec: FakeSpec) => {
    const target = specTarget(doc, spec);
    return target !== null && credentials.has(target);
  };
  const detached: FakeDetached[] = [];
  for (const { node: user } of walk(doc.nodes)) {
    if (subtree.has(user.id)) continue;
    const change: FakeDetached = { id: user.id, credential: null, settings: [] };
    if (pointsInside(user.credential)) {
      change.credential = user.credential;
      user.credential = { mode: "prompt" };
    }
    for (const { path, spec } of embeddedSpecs(user.settings)) {
      if (!pointsInside(spec)) continue;
      const before = structuredClone(valueAt(user.settings, path) ?? null);
      setValueAt(user.settings, path, PROMPT_VALUE);
      change.settings.push({ path, before, after: PROMPT_VALUE });
    }
    if (change.credential || change.settings.length > 0) detached.push(change);
  }
  return detached;
}

/**
 * Puts a deleted node back, as `restore_deleted`: a detached credential
 * goes back only if it is still as the deletion left it, otherwise the node
 * is listed as kept; a node deleted meanwhile is neither. A refusal that can
 * be fixed (a sibling took the name) leaves the deletion undoable until it
 * expires.
 */
export function undoDelete(state: FakeBackendState, undoId: string): RestoreOutcome {
  const pending =
    state.pendingDeletes[undoId] ?? fail("notFound", "l'eliminazione non si può più annullare");
  const doc = openDocument(state);
  const siblings = childrenOf(doc, pending.parent);
  checkName(siblings, pending.node.name);
  delete state.pendingDeletes[undoId];
  siblings.splice(Math.min(pending.index, siblings.length), 0, pending.node);
  const kept: string[] = [];
  for (const change of pending.detached) {
    const user = [...walk(doc.nodes)].find(({ node }) => node.id === change.id)?.node;
    if (user && !restore(user, change)) kept.push(user.id);
  }
  return {
    id: pending.node.id,
    path: pathOf(doc, pending.node.id),
    kept: kept.map((id) => ({ id, path: pathOf(doc, id) })),
  };
}

/** Gives back what is still as the deletion left it; whether everything went back. */
function restore(user: FakeNode, change: FakeDetached): boolean {
  let all = true;
  if (change.credential) {
    if (user.credential.mode === "prompt") user.credential = change.credential;
    else all = false;
  }
  for (const { path, before, after } of change.settings) {
    if (JSON.stringify(valueAt(user.settings, path)) === JSON.stringify(after)) {
      setValueAt(user.settings, path, before);
    } else {
      all = false;
    }
  }
  return all;
}

/** Forgets the deletions that can no longer be undone. */
export function expireDeletions(state: FakeBackendState): void {
  for (const [undoId, pending] of Object.entries(state.pendingDeletes)) {
    if (pending.expiresAt <= state.clock()) delete state.pendingDeletes[undoId];
  }
}
