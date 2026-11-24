import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import { useWindowActions } from "../../app/windowActions";
import { FilterChip } from "../../components/FilterChip";
import { GlassPanel } from "../../components/GlassPanel";
import type { VaultBackend } from "../../components/SecretBadge";
import { TextField } from "../../components/TextField";
import { visibleText } from "../../components/visibleText";
import { interpolate } from "../../i18n/format";
import { texts } from "../../i18n/it";
import { presentFailure } from "../../ipc/errors";
import { searchQuery } from "../../ipc/queries";
import { type TreeFilter, useWorkspace } from "../../state/workspace";
import { AddMenu } from "./AddMenu";
import { MoveToDialog } from "./MoveToDialog";
import { NodeContextMenu } from "./NodeContextMenu";
import { NodeTree } from "./NodeTree";
import styles from "./Sidebar.module.css";
import { SidebarTools } from "./SidebarTools";
import { TreeSkeleton } from "./TreeSkeleton";
import { nodeCount, visibleIds } from "./treeModel";
import { useDebounced } from "./useDebounced";
import { useTreeIndex } from "./useTreeIndex";
import { VaultFooter } from "./VaultFooter";

const FILTERS: readonly TreeFilter[] = ["all", "ssh", "rdp", "credentials", "favorites"];

/** How long the search waits for the typing to pause. */
const SEARCH_DELAY_MS = 150;

/**
 * The sidebar (inventario §5.1, DEC-050): the search (name, host, tag, by the
 * backend) with the "+" of the new nodes and the import, the filters, the
 * tree with its context menu and empty states, the footer of the vault with
 * "Valida" and "Preferenze".
 */
export function Sidebar({ backend }: { backend: VaultBackend }) {
  const t = texts.tree;
  const actions = useWindowActions();
  const tree = useTreeIndex();
  const query = useWorkspace((state) => state.query);
  const setQuery = useWorkspace((state) => state.setQuery);
  const filter = useWorkspace((state) => state.filter);
  const setFilter = useWorkspace((state) => state.setFilter);
  const searchRequests = useWorkspace((state) => state.searchRequests);
  const searchField = useRef<HTMLInputElement>(null);
  const [menu, setMenu] = useState<{ id: string; position: { x: number; y: number } }>();
  const [moving, setMoving] = useState<string>();

  // The backend searches once the user pauses typing.
  const typed = useDebounced(query.trim(), SEARCH_DELAY_MS);
  // An empty search shows the whole tree at once.
  const searched = query.trim() === "" ? "" : typed;
  const searching = searched !== "";
  const search = useQuery({
    ...searchQuery(searched),
    enabled: searching,
    placeholderData: keepPreviousData,
  });
  const found = useMemo(
    () => (searching && search.data ? new Set(search.data) : null),
    [searching, search.data],
  );
  const index = tree.data;
  const visible = useMemo(
    () => (index ? visibleIds(index, filter, searching ? (found ?? new Set()) : null) : null),
    [index, filter, searching, found],
  );

  // ⌘F and "Cerca" of the menu.
  useEffect(() => {
    if (searchRequests > 0) searchField.current?.focus();
  }, [searchRequests]);

  const menuEntry = menu && index?.entries.get(menu.id);

  return (
    <GlassPanel as="aside" className={styles.sidebar} aria-label={t.sidebar}>
      <div className={styles.head}>
        <div className={styles.searchRow}>
          <TextField
            ref={searchField}
            type="search"
            variant="search"
            label={t.search}
            placeholder={t.searchPlaceholder}
            aria-keyshortcuts="Meta+F"
            className={styles.searchField}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <AddMenu />
        </div>
        <div role="group" aria-label={t.filters.label} className={styles.filters}>
          {FILTERS.map((option) => (
            <FilterChip
              key={option}
              active={filter === option}
              onToggle={() => setFilter(option)}
              {...(option === "favorites" && { label: t.filters.favoritesLabel })}
            >
              {t.filters[option]}
            </FilterChip>
          ))}
        </div>
      </div>
      <div className={styles.body}>
        {index === undefined ? (
          <TreeSkeleton />
        ) : index.rootIds.length === 0 ? (
          <p className={styles.empty}>{t.empty}</p>
        ) : searching && search.isError ? (
          <p className={styles.empty} role="alert">
            {interpolate(t.searchFailed, {
              message: visibleText(presentFailure(search.error).message),
            })}
          </p>
        ) : visible !== null && visible.size === 0 ? (
          <p className={styles.empty} role="status">
            {searching ? t.noResults : t.noFilterResults}
          </p>
        ) : (
          <NodeTree
            index={index}
            visible={visible}
            query={searched}
            backend={backend}
            onConnect={actions.connect}
            onMove={actions.move}
            onMoveBy={(id, step) => actions.moveBy(id, step, visible)}
            onContextMenu={(id, position) => setMenu({ id, position })}
          />
        )}
      </div>
      <div className={styles.foot}>
        <VaultFooter nodeCount={index ? nodeCount(index) : undefined} />
        <SidebarTools />
      </div>
      {menu && index && menuEntry && (
        <NodeContextMenu
          entry={menuEntry}
          index={index}
          visible={visible}
          position={menu.position}
          onMoveTo={setMoving}
          onClose={() => setMenu(undefined)}
        />
      )}
      {index && <MoveToDialog index={index} id={moving} onClose={() => setMoving(undefined)} />}
    </GlassPanel>
  );
}
