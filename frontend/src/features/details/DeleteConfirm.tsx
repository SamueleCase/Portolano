import { useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";

import { reportFailure } from "../../app/failures";
import type { DeletePreview } from "../../bindings";
import { Button } from "../../components/Button";
import { Card } from "../../components/Card";
import { Dialog } from "../../components/Dialog";
import { visibleText } from "../../components/visibleText";
import { VisuallyHidden } from "../../components/VisuallyHidden";
import { interpolate, plural } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { useDeleteNode, useUndoDelete } from "../../ipc/mutations";
import { treeQuery } from "../../ipc/queries";
import { showToast } from "../../state/toasts";
import { useWorkspace } from "../../state/workspace";
import { indexTree, isWithin, readablePath } from "../tree/treeModel";
import { useGoToNode } from "../tree/useGoToNode";
import { useDeleteConfirm } from "./deleteStore";
import { deleteUsage } from "./deleteUsage";
import styles from "./Details.module.css";

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
 * The confirmation of a deletion (inventario §5.11, generic): who refers to
 * what is deleted (each row goes to the node), how many receive it by
 * inheritance, the content and the secrets of a folder, "Elimina comunque"
 * when used. Enter means "Annulla", the default action. Once deleted,
 * "Annulla" of the toast puts everything back for 10 seconds (`undo_delete`);
 * then the secrets leave the vault.
 */
export function DeleteConfirm() {
  const t = texts.details.remove;
  const queryClient = useQueryClient();
  const preview = useDeleteConfirm((state) => state.preview);
  const name = useDeleteConfirm((state) => state.name);
  const dismiss = useDeleteConfirm((state) => state.dismiss);
  const deleteNode = useDeleteNode();
  const { mutateAsync: undo } = useUndoDelete();
  const goToNode = useGoToNode();
  const cancel = useRef<HTMLButtonElement>(null);

  const view = queryClient.getQueryData(treeQuery.queryKey);
  const usage = preview ? deleteUsage(preview, view ? indexTree(view) : undefined) : undefined;
  const used = usage !== undefined && (usage.rows.length > 0 || usage.inherited > 0);
  const consequences = [
    usage?.credentialsUsed ? t.credentialText : "",
    usage?.otherReferences ? t.referencesText : "",
  ].filter((text) => text !== "");

  const undoDeletion = (undoId: string) =>
    void undo(undoId)
      .then((restored) => {
        showToast({ kind: "success", text: interpolate(texts.app.toasts.restored, { name }) });
        if (restored.kept.length > 0) {
          showToast({
            kind: "info",
            text: interpolate(plural(restored.kept.length, texts.app.toasts.kept), {
              paths: restored.kept.map((node) => readablePath(node.path)).join(", "),
            }),
          });
        }
      })
      .catch((failure: unknown) => reportFailure(failure, queryClient));

  const confirm = async () => {
    if (!preview) return;
    const { selectedId, clearSelection } = useWorkspace.getState();
    try {
      const result = await deleteNode.mutateAsync({
        id: preview.id,
        options: { recursive: preview.children > 0, force: used },
      });
      // The detail of a deleted node has nothing to show.
      if (selectedId !== null && view && isWithin(indexTree(view), selectedId, preview.id)) {
        clearSelection();
      }
      dismiss();
      showToast({
        kind: "undo",
        text: interpolate(texts.app.toasts.deleted, { name }),
        action: { label: texts.app.toasts.undo, onAction: () => undoDeletion(result.undoId) },
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
      {usage && used && (
        <Card tone="warning" className={styles.usedBy}>
          {usage.rows.length > 0 && (
            <>
              <p className={styles.usedByTitle}>{plural(usage.rows.length, t.usedBy)}</p>
              <ul className={styles.usedByList}>
                {usage.rows.map((row) => (
                  <li key={row.key}>
                    <button
                      type="button"
                      className={styles.usageRow}
                      onClick={() => {
                        dismiss();
                        goToNode(row.id);
                      }}
                    >
                      <span>{visibleText(readablePath(row.path))}</span>
                      <VisuallyHidden>, </VisuallyHidden>
                      <span className={styles.relation}>{row.relation}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </>
          )}
          {usage.inherited > 0 && (
            <p className={styles.inheritedBy}>{plural(usage.inherited, t.inheritedBy)}</p>
          )}
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
