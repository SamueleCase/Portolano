import { type KeyboardEvent, useRef } from "react";

import { interpolate, plural } from "../i18n/format";
import { texts } from "../i18n/it";
import { classNames } from "./classNames";
import { type DotState, StatusDot } from "./StatusDot";
import { tabElementId } from "./tabIds";
import styles from "./Tabs.module.css";
import { visibleText } from "./visibleText";

/**
 * The kinds of work tab (inventario §4.8). The session kinds follow the
 * session model of SPEC-UI §5.1, so an embedded RDP panel is one more kind.
 */
export type WorkTabKind = "detail" | "validation" | "ssh" | "rdp" | "external";

/** A tab of the work area. */
export interface WorkTab {
  id: string;
  kind: WorkTabKind;
  /** The node name (untrusted) or "Validazione". */
  title: string;
  dot: DotState;
  /** The detail tab cannot be closed. */
  closable: boolean;
  /** The number of problems, for the validation tab. */
  problems?: number;
}

interface TabsProps {
  label?: string;
  tabs: readonly WorkTab[];
  selectedId: string;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  /** The id of the panel the tabs control. */
  panelId: string;
}

function kindLabel(tab: WorkTab): string {
  if (tab.kind === "validation") {
    return plural(tab.problems ?? 0, texts.components.tabs.problems);
  }
  return texts.components.tabs.kinds[tab.kind];
}

/**
 * The tab bar of the work area: a `tablist` where the arrows move and select,
 * Delete closes the focused tab, and "×" closes with the pointer.
 */
export function Tabs({
  label = texts.components.tabs.label,
  tabs,
  selectedId,
  onSelect,
  onClose,
  panelId,
}: TabsProps) {
  const elements = useRef(new Map<string, HTMLButtonElement>());

  const keyDown = (event: KeyboardEvent, index: number) => {
    const tab = tabs[index];
    if ((event.key === "Delete" || event.key === "Backspace") && tab?.closable) {
      event.preventDefault();
      onClose(tab.id);
      return;
    }
    const target = {
      ArrowRight: (index + 1) % tabs.length,
      ArrowLeft: (index - 1 + tabs.length) % tabs.length,
      Home: 0,
      End: tabs.length - 1,
    }[event.key];
    const next = target === undefined ? undefined : tabs[target];
    if (!next) return;
    event.preventDefault();
    onSelect(next.id);
    elements.current.get(next.id)?.focus();
  };

  return (
    <div role="tablist" aria-label={label} className={styles.bar}>
      {tabs.map((tab, index) => {
        const selected = tab.id === selectedId;
        const title = visibleText(tab.title);
        return (
          <div
            key={tab.id}
            role="presentation"
            className={classNames(styles.tab, selected && styles.selected)}
          >
            <button
              ref={(element) => {
                if (element) elements.current.set(tab.id, element);
                else elements.current.delete(tab.id);
              }}
              id={tabElementId(panelId, tab.id)}
              type="button"
              role="tab"
              aria-selected={selected}
              aria-controls={panelId}
              aria-keyshortcuts={tab.closable ? "Delete" : undefined}
              tabIndex={selected ? 0 : -1}
              className={styles.trigger}
              onClick={() => onSelect(tab.id)}
              onKeyDown={(event) => keyDown(event, index)}
            >
              <StatusDot
                state={tab.dot}
                decorative={tab.kind === "detail" || tab.kind === "validation"}
              />{" "}
              <span className={styles.title}>{title}</span>{" "}
              <span className={styles.kind}>{kindLabel(tab)}</span>
            </button>
            {tab.closable && (
              // Only for the pointer: a `tablist` may contain only tabs, and
              // the keyboard closes the focused tab with Delete.
              <button
                type="button"
                tabIndex={-1}
                aria-hidden="true"
                className={styles.close}
                title={interpolate(texts.components.tabs.close, { name: title })}
                onClick={() => onClose(tab.id)}
              >
                ×
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
