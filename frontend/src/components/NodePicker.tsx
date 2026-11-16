import { type CSSProperties, type KeyboardEvent, useId, useState } from "react";

import { interpolate, plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { Button } from "./Button";
import { classNames } from "./classNames";
import { Dialog } from "./Dialog";
import { HighlightedText } from "./HighlightedText";
import { NodeTypeIcon } from "./icons/NodeTypeIcon";
import styles from "./NodePicker.module.css";
import { folderRows, nodeRows, type PickerNode, type PickerRow } from "./nodePickerRows";
import { visibleText } from "./visibleText";

export type { PickerNode } from "./nodePickerRows";

interface CommonProps {
  open: boolean;
  onClose: () => void;
  /** "Scegli un jump host", "Scegli una credenziale", "Scegli la cartella". */
  title: string;
  /** The filter applied, such as "Solo SSH" (inventario §5.8). */
  filterLabel: string;
  nodes: readonly PickerNode[];
  /** Why some nodes are missing, such as "web01 è escluso perché…". */
  note?: string | undefined;
}

/** Chooses a connection or a credential; the folders only head their content (the default). */
interface NodeChoice {
  choose?: "nodes";
  onChoose: (id: string) => void;
  /** Only for `choose="folders"`: here it would be ignored. */
  exclude?: never;
}

/**
 * Chooses a folder, empty ones included, or the root of the document: the
 * first row, which `onChoose` receives as `null`, like the `parent` of the
 * commands. The other nodes of `nodes` are not shown.
 */
interface FolderChoice {
  choose: "folders";
  onChoose: (id: string | null) => void;
  /**
   * A node shown but not choosable together with its descendants, such as the
   * folder being moved, which cannot go inside itself.
   */
  exclude?: string | undefined;
}

type NodePickerProps = CommonProps & (NodeChoice | FolderChoice);

/** The indentation of a folder by its level, in the stylesheet. */
function depthStyle(row: PickerRow): CSSProperties & { "--picker-depth": number } {
  return { "--picker-depth": row.depth };
}

/**
 * The picker of a node (inventario §5.8): the tree filters while typing, the
 * arrows move among the choosable rows and Enter chooses.
 */
export function NodePicker(props: NodePickerProps) {
  const { open, onClose, title, filterLabel, nodes, note } = props;
  const t = texts.components.nodePicker;
  const folders = props.choose === "folders";
  const id = useId();
  const [search, setSearch] = useState("");
  const [activeKey, setActiveKey] = useState<string>();
  const rows =
    props.choose === "folders"
      ? folderRows(nodes, search, t.root, props.exclude)
      : nodeRows(nodes, search);
  const choosable = rows.filter((row) => !row.heading && !row.disabled);
  const active = choosable.find((row) => row.key === activeKey) ?? choosable[0];
  const optionId = (row: PickerRow) => `${id}-${row.key}`;

  const choose = (row: PickerRow) => {
    if (props.choose === "folders") {
      props.onChoose(row.value);
    } else if (row.value !== null) {
      // Choosing nodes there is no root row, so the value is always an id.
      props.onChoose(row.value);
    }
  };

  const keyDown = (event: KeyboardEvent) => {
    const index = active ? choosable.indexOf(active) : -1;
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (step !== undefined) {
      event.preventDefault();
      const next = choosable[Math.min(choosable.length - 1, Math.max(0, index + step))];
      if (next) setActiveKey(next.key);
    } else if (event.key === "Enter" && active) {
      event.preventDefault();
      choose(active);
    }
  };

  const confirmLabel = (row: PickerRow | undefined) => {
    if (!row) return t.choose;
    if (row.value === null) return t.chooseRoot;
    return interpolate(t.chooseNamed, { name: visibleText(row.name) });
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
          <Button variant="primary" disabled={!active} onClick={() => active && choose(active)}>
            {confirmLabel(active)}
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
          aria-activedescendant={active ? optionId(active) : undefined}
          className={styles.search}
          placeholder={t.search}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          onKeyDown={keyDown}
        />
        <span className={styles.filter}>{filterLabel}</span>
      </div>
      <ul id={`${id}-list`} role="listbox" aria-label={title} className={styles.list}>
        {rows.map((row) =>
          row.heading ? (
            <li key={row.key} role="presentation" className={styles.folder}>
              <span aria-hidden="true">▾</span> {visibleText(row.name)}
            </li>
          ) : (
            <li
              key={row.key}
              id={optionId(row)}
              role="option"
              aria-selected={row.key === active?.key}
              aria-disabled={row.disabled || undefined}
              className={classNames(
                styles.node,
                folders && styles.nested,
                row.key === active?.key && styles.active,
                row.disabled && styles.excluded,
              )}
              style={folders ? depthStyle(row) : undefined}
              onClick={() => !row.disabled && setActiveKey(row.key)}
              onDoubleClick={() => !row.disabled && choose(row)}
            >
              <NodeTypeIcon kind={row.kind} size="list" />
              <span className={styles.name}>
                <HighlightedText text={row.name} match={search.trim() || undefined} />
              </span>
              {row.host && <span className={styles.host}>{visibleText(row.host)}</span>}
            </li>
          ),
        )}
      </ul>
      <p className={styles.summary} role="status">
        {choosable.length > 0
          ? plural(choosable.length, t.results)
          : folders
            ? t.emptyFolders
            : t.empty}
        {note && ` ${note}`}
      </p>
    </Dialog>
  );
}
