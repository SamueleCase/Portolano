import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import { useWindowActions } from "../../app/windowActions";
import { FilterChip } from "../../components/FilterChip";
import { GlassPanel } from "../../components/GlassPanel";
import type { VaultBackend } from "../../components/SecretBadge";
import { TextField } from "../../components/TextField";
import { texts } from "../../i18n/it";
import { searchQuery } from "../../ipc/queries";
import { type TreeFilter, useWorkspace } from "../../state/workspace";
import { MoveToDialog } from "./MoveToDialog";
import { NodeContextMenu } from "./NodeContextMenu";
import { NodeTree } from "./NodeTree";
import styles from "./Sidebar.module.css";
import { TreeSkeleton } from "./TreeSkeleton";
import { nodeCount, visibleIds } from "./treeModel";
import { useTreeIndex } from "./useTreeIndex";
import { VaultFooter } from "./VaultFooter";

const FILTERS: readonly TreeFilter[] = ["all", "ssh", "rdp", "credentials", "favorites"];

/**
 * The sidebar (inventario §5.1): the search (name, host, tag, by the backend)
 * and the filters, the tree with its context menu and empty states, the
 * footer of the vault.
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

  const searching = query.trim() !== "";
  const search = useQuery({
    ...searchQuery(query.trim()),
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
        <TextField
          ref={searchField}
          type="search"
          variant="search"
          label={t.search}
          placeholder={t.searchPlaceholder}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
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
        ) : visible !== null && visible.size === 0 ? (
          <p className={styles.empty} role="status">
            {t.noResults}
          </p>
        ) : (
          <NodeTree
            index={index}
            visible={visible}
            query={query.trim()}
            backend={backend}
            onConnect={actions.connect}
            onMove={actions.move}
            onContextMenu={(id, position) => setMenu({ id, position })}
          />
        )}
      </div>
      <VaultFooter nodeCount={index ? nodeCount(index) : 0} />
      {menu && menuEntry && (
        <NodeContextMenu
          entry={menuEntry}
          position={menu.position}
          onMoveTo={setMoving}
          onClose={() => setMenu(undefined)}
        />
      )}
      {index && <MoveToDialog index={index} id={moving} onClose={() => setMoving(undefined)} />}
    </GlassPanel>
  );
}
