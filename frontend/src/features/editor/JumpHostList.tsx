import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";

import { Button } from "../../components/Button";
import { NodeTypeIcon } from "../../components/icons/NodeTypeIcon";
import { RouteChain } from "../../components/RouteChain";
import { SortableList } from "../../components/SortableList";
import { SourceChip } from "../../components/SourceChip";
import { TextField } from "../../components/TextField";
import { visibleText } from "../../components/visibleText";
import { texts } from "../../i18n/it";
import { treeQuery } from "../../ipc/queries";
import { TreeNodePicker } from "../dialogs/TreeNodePicker";
import { type IndexedNode, indexedNodes } from "../dialogs/treeIndex";
import { summaryHost } from "../tree/treeModel";
import { useEditorFields } from "./editorContext";
import styles from "./Editor.module.css";
import type { JumpRow } from "./editorValues";
import { GroupError } from "./GroupError";

/**
 * The jump hosts in the order they are crossed (inventario §5.5): SSH
 * connections of the tree or free addresses, reorderable by drag or with
 * Alt+↑/↓, with the chain "tu → … → node" under them.
 */
export function JumpHostList() {
  const t = texts.editor.ssh;
  const { values, set, errorOf, nodeId, newRowId } = useEditorFields();
  const [picking, setPicking] = useState(false);
  const [typing, setTyping] = useState(false);
  const [address, setAddress] = useState("");
  const labelId = useId();
  const errorId = useId();
  const { data: tree } = useQuery(treeQuery);
  const entries = tree ? indexedNodes(tree) : [];
  const jumps = values.jumpHosts;
  const nodeOf = (jump: JumpRow): IndexedNode | undefined =>
    "ref" in jump ? entries.find((entry) => entry.node.id === jump.ref) : undefined;
  const nameOf = (jump: JumpRow) =>
    "ref" in jump ? (nodeOf(jump)?.node.name ?? t.missingJump) : jump.address;

  const append = (jump: JumpRow) => set("jumpHosts", [...jumps, jump]);
  const addAddress = () => {
    if (address.trim() !== "") append({ id: newRowId(), address: address.trim() });
    setAddress("");
    setTyping(false);
  };
  const error = errorOf("jumpHosts");

  return (
    <div
      className={styles.group}
      role="group"
      aria-labelledby={labelId}
      aria-describedby={error ? errorId : undefined}
    >
      <span id={labelId} className={styles.fieldLabel}>
        {t.jumps}
      </span>
      <SortableList
        label={t.jumps}
        items={jumps}
        itemKey={(jump) => jump.id}
        itemName={nameOf}
        onMove={(from, to) => {
          const reordered = [...jumps];
          const [moved] = reordered.splice(from, 1);
          if (moved) reordered.splice(to, 0, moved);
          set("jumpHosts", reordered);
        }}
        onRemove={(index) =>
          set(
            "jumpHosts",
            jumps.filter((_, position) => position !== index),
          )
        }
        renderItem={(jump) => {
          const node = nodeOf(jump);
          return "ref" in jump ? (
            <span className={styles.jump}>
              <NodeTypeIcon kind="ssh" size="list" />
              <span className={styles.jumpName}>{visibleText(nameOf(jump))}</span>
              {node && <span className={styles.jumpPath}>{visibleText(node.path)}</span>}
              <SourceChip source="tree" />
            </span>
          ) : (
            <span className={styles.jump}>
              <span className={styles.jumpAddress}>{visibleText(jump.address)}</span>
              <SourceChip source="address" />
            </span>
          );
        }}
        footer={
          <>
            <Button variant="add" onClick={() => setPicking(true)}>
              {t.fromTree}
            </Button>
            <Button variant="add" onClick={() => setTyping(true)}>
              {t.freeAddress}
            </Button>
          </>
        }
      />
      {typing && (
        <div className={styles.addressRow}>
          <TextField
            label={t.address}
            mono
            autoFocus
            value={address}
            className={styles.grow}
            onChange={(event) => setAddress(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                addAddress();
              }
            }}
          />
          <Button onClick={addAddress}>{t.addAddress}</Button>
        </div>
      )}
      {jumps.length > 0 && (
        <RouteChain
          variant="editor"
          stops={[
            ...jumps.map((jump) => ({
              label: "ref" in jump ? nameOf(jump) : summaryHost(jump.address),
            })),
            { label: values.name || texts.editor.untitled },
          ]}
        />
      )}
      <GroupError id={errorId} text={error} />
      <TreeNodePicker
        open={picking}
        purpose="jumpHost"
        exclude={nodeId}
        onClose={() => setPicking(false)}
        onChoose={(id) => {
          append({ id: newRowId(), ref: id });
          setPicking(false);
        }}
      />
    </div>
  );
}
