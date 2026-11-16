import { useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";

import { reportFailure } from "../../app/failures";
import type { DeletePreview } from "../../bindings";
import { Button } from "../../components/Button";
import { Card } from "../../components/Card";
import { Dialog } from "../../components/Dialog";
import { visibleText } from "../../components/visibleText";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { useDeleteNode, useUndoDelete } from "../../ipc/mutations";
import { treeQuery } from "../../ipc/queries";
import { showToast } from "../../state/toasts";
import { useWorkspace } from "../../state/workspace";
import { indexTree, isWithin, readablePath } from "../tree/treeModel";
import { useDeleteConfirm } from "./deleteStore";
import styles from "./Details.module.css";

/** A row of "È usata da N nodi": who, and how. */
interface UsageRow {
  key: string;
  path: string;
  relation: string;
}

/** The users of what is deleted (DEC-018.1 and ambiguity 13): direct references, then heirs. */
function usageRows(preview: DeletePreview): UsageRow[] {
  const t = texts.details.remove;
  const credential = preview.node_type.type === "credential";
  return [
    ...preview.references.map((reference) => ({
      key: `${reference.from}-${reference.target}`,
      path: reference.from_path,
      relation:
        credential && reference.target === preview.id
          ? t.linked
          : interpolate(t.references, { target: readablePath(reference.target_path) }),
    })),
    ...preview.inherited.map((node) => ({
      key: `${node.id}-inherited`,
      path: node.path,
      relation: t.inherited,
    })),
  ];
}

function title(preview: DeletePreview, name: string): string {
  const t = texts.details.remove;
  const template =
    preview.node_type.type === "credential"
      ? t.titleCredential
      : preview.node_type.type === "folder"
        ? t.titleFolder
        : t.title;
  return interpolate(template, { name });
}

/**
 * The confirmation of a deletion (inventario §5.11, generic): who uses what is
 * deleted, the content and the secrets of a folder, "Elimina comunque" when
 * used. Enter means "Annulla", the default action. Once deleted, "Annulla" of
 * the toast puts everything back for 10 seconds (`undo_delete`); then the
 * secrets leave the vault.
 */
export function DeleteConfirm() {
  const t = texts.details.remove;
  const queryClient = useQueryClient();
  const preview = useDeleteConfirm((state) => state.preview);
  const name = useDeleteConfirm((state) => state.name);
  const dismiss = useDeleteConfirm((state) => state.dismiss);
  const deleteNode = useDeleteNode();
  const { mutateAsync: undo } = useUndoDelete();
  const cancel = useRef<HTMLButtonElement>(null);

  const rows = preview ? usageRows(preview) : [];
  const used = rows.length > 0;
  const credential = preview?.node_type.type === "credential";
  // Credentials leave their users "ask at launch"; other references stay dangling.
  const consequences = preview
    ? [
        credential || preview.inherited.length > 0 ? t.credentialText : "",
        !credential && preview.references.length > 0 ? t.referencesText : "",
      ].filter((text) => text !== "")
    : [];

  const confirm = async () => {
    if (!preview) return;
    // Closing the selection first: the detail of a deleted node has nothing to show.
    const index = queryClient.getQueryData(treeQuery.queryKey);
    const { selectedId, clearSelection } = useWorkspace.getState();
    try {
      const result = await deleteNode.mutateAsync({
        id: preview.id,
        options: { recursive: preview.children > 0, force: used },
      });
      if (selectedId !== null && index && isWithin(indexTree(index), selectedId, preview.id)) {
        clearSelection();
      }
      dismiss();
      showToast({
        kind: "undo",
        text: interpolate(texts.app.toasts.deleted, { name }),
        action: {
          label: texts.app.toasts.undo,
          onAction: () =>
            void undo(result.undoId)
              .then((restored) => {
                showToast({
                  kind: "success",
                  text: interpolate(texts.app.toasts.restored, { name }),
                });
                if (restored.kept.length > 0) {
                  showToast({
                    kind: "info",
                    text: interpolate(plural(restored.kept.length, texts.app.toasts.kept), {
                      paths: restored.kept.map((node) => readablePath(node.path)).join(", "),
                    }),
                  });
                }
              })
              .catch((failure: unknown) => reportFailure(failure, queryClient)),
        },
      });
    } catch (failure) {
      dismiss();
      reportFailure(failure, queryClient);
    }
  };

  return (
    <Dialog
      open={preview !== null}
      onClose={dismiss}
      title={preview ? title(preview, name) : undefined}
      subtitle={preview ? readablePath(preview.path) : undefined}
      width="medium"
      initialFocus={cancel}
      // Enter is "Annulla" here (SPEC-UI §5.5).
      onSubmit={dismiss}
      footer={
        <>
          <Button ref={cancel} type="submit" className={styles.defaultCancel}>
            {t.cancel}
          </Button>
          <Button variant="danger" loading={deleteNode.isPending} onClick={() => void confirm()}>
            {used ? t.deleteAnyway : t.delete}
          </Button>
        </>
      }
    >
      {preview && used && (
        <Card tone="warning" className={styles.usedBy}>
          <p className={styles.usedByTitle}>{plural(rows.length, t.usedBy)}</p>
          <ul className={styles.usedByList}>
            {rows.map((row) => (
              <li key={row.key} className={styles.usedByRow}>
                <span>{visibleText(readablePath(row.path))}</span>
                <span className={styles.relation}>{row.relation}</span>
              </li>
            ))}
          </ul>
        </Card>
      )}
      {preview && preview.node_type.type === "folder" && preview.descendants > 0 && (
        <p className={styles.dialogText}>
          {interpolate(t.folder, {
            elements: plural(preview.descendants, t.elements),
            secrets: plural(preview.secrets, t.secrets),
          })}
        </p>
      )}
      {preview && (
        <p className={styles.dialogText}>
          {[used ? consequences.join(" ") : t.unused, t.undoHint].join(" ")}
        </p>
      )}
    </Dialog>
  );
}
