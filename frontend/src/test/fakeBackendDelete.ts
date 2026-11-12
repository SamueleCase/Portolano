import type {
  DeleteOptionsInput_Deserialize,
  DeletePreviewView,
  DeleteResult,
  UndoOutcome,
} from "../bindings";
import {
  type FakeBackendState,
  type FakeNode,
  type FakeSpec,
  checkName,
  childrenOf,
  fail,
  findNode,
  newId,
  openDocument,
  pathOf,
  subtreeIds,
  walk,
} from "./fakeBackendModel";
import { referencesTo, usageOf } from "./fakeBackendViews";

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

export function previewDelete(state: FakeBackendState, id: string): DeletePreviewView {
  const doc = openDocument(state);
  const { node } = findNode(doc, id);
  const subtree = subtreeIds(node);
  const inherited = [...walk([node])]
    .filter(({ node: inner }) => inner.kind === "credential")
    .flatMap(({ node: credential }) => usageOf(doc, credential).inherited)
    .filter((user) => !subtree.has(user.id));
  return {
    preview: {
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
      secrets: [...walk([node])].reduce(
        (count, { node: inner }) => count + Object.keys(inner.secrets).length,
        0,
      ),
      references: externalReferences(state, node),
    },
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
      {
        kind: "referenced",
        references,
      },
    );
  }
  const detached: { id: string; credential: FakeSpec }[] = [];
  for (const reference of references) {
    const from = findNode(doc, reference.from).node;
    if (from.credential.mode === "byId" || from.credential.mode === "byName") {
      detached.push({ id: from.id, credential: from.credential });
      from.credential = { mode: "prompt" };
    }
  }
  const siblings = parent ? parent.children : doc.nodes;
  const index = siblings.indexOf(node);
  siblings.splice(index, 1);
  const undoId = newId(state);
  state.pendingDeletes[undoId] = { node, parent: parent?.id ?? null, index, detached };
  return {
    outcome: {
      removed_nodes: subtreeIds(node).size,
      detached: detached.map((entry) => entry.id),
      remaining_references: references.filter(
        (reference) => !detached.some((entry) => entry.id === reference.from),
      ),
      secrets_deleted: 0,
      cleanup_failures: [],
    },
    undoId,
  };
}

export function undoDelete(state: FakeBackendState, undoId: string): UndoOutcome {
  const pending =
    state.pendingDeletes[undoId] ?? fail("notFound", "l'eliminazione non si può più annullare");
  delete state.pendingDeletes[undoId];
  const doc = openDocument(state);
  const siblings = childrenOf(doc, pending.parent);
  checkName(siblings, pending.node.name);
  siblings.splice(Math.min(pending.index, siblings.length), 0, pending.node);
  for (const { id, credential } of pending.detached) {
    findNode(doc, id).node.credential = credential;
  }
  return { id: pending.node.id, path: pathOf(doc, pending.node.id) };
}
