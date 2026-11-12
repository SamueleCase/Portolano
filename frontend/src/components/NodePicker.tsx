import { type KeyboardEvent, useId, useState } from "react";

import { interpolate, plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { Button } from "./Button";
import { classNames } from "./classNames";
import { Dialog } from "./Dialog";
import { HighlightedText } from "./HighlightedText";
import { NodeTypeIcon } from "./icons/NodeTypeIcon";
import type { NodeKind } from "./nodeKind";
import styles from "./NodePicker.module.css";
import { visibleText } from "./visibleText";

/** A node of the picker, in tree order (folders before their content). */
export interface PickerNode {
  id: string;
  name: string;
  kind: NodeKind;
  depth: number;
  host?: string | undefined;
  /** Shown but not choosable, such as the connection being edited. */
  excluded?: boolean;
}

interface NodePickerProps {
  open: boolean;
  onClose: () => void;
  onChoose: (id: string) => void;
  /** "Scegli un jump host", "Scegli una credenziale", "Scegli la cartella". */
  title: string;
  /** The filter applied, such as "Solo SSH" (inventario §5.8). */
  filterLabel: string;
  nodes: readonly PickerNode[];
  /** Why some nodes are missing, such as "web01 è escluso perché…". */
  note?: string | undefined;
}

function contains(node: PickerNode, search: string): boolean {
  const needle = search.trim().toLocaleLowerCase("it");
  return [node.name, node.host ?? ""].some((text) => text.toLocaleLowerCase("it").includes(needle));
}

/** The nodes that match `search`, with the folders that lead to them. */
function visibleNodes(nodes: readonly PickerNode[], search: string): PickerNode[] {
  const shown = new Set<number>();
  nodes.forEach((node, index) => {
    if (node.kind === "folder" || !contains(node, search)) return;
    shown.add(index);
    let depth = node.depth;
    for (let parent = index - 1; parent >= 0 && depth > 0; parent--) {
      const candidate = nodes[parent];
      if (candidate && candidate.depth < depth) {
        shown.add(parent);
        depth = candidate.depth;
      }
    }
  });
  return nodes.filter((_, index) => shown.has(index));
}

/**
 * The picker of a node (inventario §5.8): the tree filters while typing, the
 * arrows move among the choosable nodes and Enter chooses.
 */
export function NodePicker({
  open,
  onClose,
  onChoose,
  title,
  filterLabel,
  nodes,
  note,
}: NodePickerProps) {
  const t = texts.components.nodePicker;
  const id = useId();
  const [search, setSearch] = useState("");
  const [activeId, setActiveId] = useState<string>();
  const shown = visibleNodes(nodes, search);
  const choosable = shown.filter((node) => node.kind !== "folder" && !node.excluded);
  const active = choosable.find((node) => node.id === activeId) ?? choosable[0];

  const keyDown = (event: KeyboardEvent) => {
    const index = active ? choosable.indexOf(active) : -1;
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (step !== undefined) {
      event.preventDefault();
      const next = choosable[Math.min(choosable.length - 1, Math.max(0, index + step))];
      if (next) setActiveId(next.id);
    } else if (event.key === "Enter" && active) {
      event.preventDefault();
      onChoose(active.id);
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={title}
      width="small"
      footer={
        <>
          <Button onClick={onClose}>{t.cancel}</Button>
          <Button
            variant="primary"
            disabled={!active}
            onClick={() => active && onChoose(active.id)}
          >
            {active ? interpolate(t.chooseNamed, { name: visibleText(active.name) }) : t.choose}
          </Button>
        </>
      }
    >
      <div className={styles.searchRow}>
        <input
          role="combobox"
          aria-label={t.search}
          aria-expanded="true"
          aria-controls={`${id}-list`}
          aria-activedescendant={active ? `${id}-${active.id}` : undefined}
          className={styles.search}
          placeholder={t.search}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          onKeyDown={keyDown}
        />
        <span className={styles.filter}>{filterLabel}</span>
      </div>
      <ul id={`${id}-list`} role="listbox" aria-label={title} className={styles.list}>
        {shown.map((node) =>
          node.kind === "folder" ? (
            <li key={node.id} role="presentation" className={styles.folder}>
              <span aria-hidden="true">▾</span> {visibleText(node.name)}
            </li>
          ) : (
            <li
              key={node.id}
              id={`${id}-${node.id}`}
              role="option"
              aria-selected={node.id === active?.id}
              aria-disabled={node.excluded || undefined}
              className={classNames(
                styles.node,
                node.id === active?.id && styles.active,
                node.excluded && styles.excluded,
              )}
              onClick={() => !node.excluded && setActiveId(node.id)}
              onDoubleClick={() => !node.excluded && onChoose(node.id)}
            >
              <NodeTypeIcon kind={node.kind} size="list" />
              <span className={styles.name}>
                <HighlightedText text={node.name} match={search.trim() || undefined} />
              </span>
              {node.host && <span className={styles.host}>{visibleText(node.host)}</span>}
            </li>
          ),
        )}
      </ul>
      <p className={styles.summary} role="status">
        {choosable.length === 0 ? t.empty : plural(choosable.length, t.results)}
        {note && ` ${note}`}
      </p>
    </Dialog>
  );
}
