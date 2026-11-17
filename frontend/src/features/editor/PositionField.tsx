import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";

import { Button } from "../../components/Button";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { FolderPicker } from "../dialogs/FolderPicker";
import { indexedNodes } from "../dialogs/treeIndex";
import { useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";

/**
 * "Posizione": the folder the node goes in, with "Cambia…" (ambiguity 10:
 * for a new node it comes from the selection of the main window). For an
 * existing node "Salva" moves it there.
 */
export function PositionField() {
  const id = useId();
  const { values, set, errorOf, nodeId } = useEditorFields();
  const [choosing, setChoosing] = useState(false);
  const { data: tree } = useQuery(treeQuery);
  const folder = tree
    ? indexedNodes(tree).find((entry) => entry.node.id === values.parent)
    : undefined;
  const shown = values.parent === null ? texts.dialogs.root : folder?.path;
  const error = errorOf("parent");
  return (
    <div className={styles.group} role="group" aria-labelledby={id}>
      <span id={id} className={styles.fieldLabel}>
        {texts.editor.general.position}
      </span>
      <div className={styles.position}>
        <span className={styles.positionPath}>{shown ? visibleText(shown) : ""}</span>
        <Button variant="small" onClick={() => setChoosing(true)}>
          {texts.editor.general.change}
        </Button>
      </div>
      {error && (
        <span className={styles.error} role="alert">
          {error}
        </span>
      )}
      <FolderPicker
        open={choosing}
        exclude={nodeId}
        onClose={() => setChoosing(false)}
        onChoose={(parent) => {
          set("parent", parent);
          setChoosing(false);
        }}
      />
    </div>
  );
}
