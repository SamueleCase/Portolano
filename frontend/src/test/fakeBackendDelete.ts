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
  type FakePendingDelete,
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
 * point to a credential of it, by id or by a name that resolves to it, as
 * `detach_credential_references`; other references, such as jump hosts,
 * stay (dangling).
 */
function detachCredentials(doc: FakeDocument, node: FakeNode): FakePendingDelete["detached"] {
  const subtree = subtreeIds(node);
  const credentials = new Set(
    [...walk([node])]
      .filter(({ node: inner }) => inner.kind === "credential")
      .map(({ node: inner }) => inner.id),
  );
  const detached: FakePendingDelete["detached"] = [];
  for (const { node: user } of walk(doc.nodes)) {
    if (subtree.has(user.id)) continue;
    const target = specTarget(doc, user.credential);
    if (target !== null && credentials.has(target)) {
      detached.push({ id: user.id, credential: user.credential });
      user.credential = { mode: "prompt" };
    }
  }
  return detached;
}

/**
 * Puts a deleted node back, as `restore_deleted`: a detached credential
 * goes back only if it is still "ask at launch", otherwise the node is
 * listed as kept. A refusal that can be fixed (a sibling took the name)
 * leaves the deletion undoable until it expires.
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
  for (const { id, credential } of pending.detached) {
    const user = [...walk(doc.nodes)].find(({ node }) => node.id === id)?.node;
    if (user?.credential.mode === "prompt") {
      user.credential = credential;
    } else {
      kept.push(id);
    }
  }
  return {
    id: pending.node.id,
    path: pathOf(doc, pending.node.id),
    kept: kept
      .filter((keptId) => [...walk(doc.nodes)].some(({ node }) => node.id === keptId))
      .map((keptId) => ({ id: keptId, path: pathOf(doc, keptId) })),
  };
}

/** Forgets the deletions that can no longer be undone. */
export function expireDeletions(state: FakeBackendState): void {
  for (const [undoId, pending] of Object.entries(state.pendingDeletes)) {
    if (pending.expiresAt <= state.clock()) delete state.pendingDeletes[undoId];
  }
}
