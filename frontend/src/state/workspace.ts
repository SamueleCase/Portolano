import { create } from "zustand";

/** The tab of the detail: always there while the document has nodes. */
export const DETAIL_TAB = "detail";

/** The tab of the validation, open after "Valida". */
export const VALIDATION_TAB = "validation";

/** The filters of the tree (inventario §4.7). */
export type TreeFilter = "all" | "ssh" | "rdp" | "credentials" | "favorites";

/** What happened to the document file (SPEC-UI §5.4). */
export type DocumentBanner =
  { kind: "modified"; fileName: string } | { kind: "missing"; fileName: string };

/**
 * The state of the main window (SPEC-UI §5.1): the selected node, the tab in
 * front, the search and the filter of the tree, the banner of the document.
 * The session tabs come from the store of the sessions; here only which tab
 * is in front. Nothing here comes from the vault: no secret, ever.
 */
export interface WorkspaceState {
  selectedId: string | null;
  /** `DETAIL_TAB`, `VALIDATION_TAB` or the id of a session. */
  activeTab: string;
  validationOpen: boolean;
  /** "Mostra comando" in the detail of the selected node. */
  commandShown: boolean;
  query: string;
  filter: TreeFilter;
  banner: DocumentBanner | null;
  /** Grows at each request to focus the search field (⌘F). */
  searchRequests: number;
  /** The menu of "+ Connessione" is open (its button, or ⌘N). */
  connectionMenuOpen: boolean;
  /** The open folders of the tree; `null` until the user opens or closes one. */
  expanded: readonly string[] | null;
  /** Grows at each `reveal`: the tree moves its focus to the node revealed. */
  revealRequests: number;
  /**
   * The summary of the connection of each session, when it opened: the title
   * of its tab once the node is gone (DEC-041.7).
   */
  sessionSummaries: Readonly<Record<string, string>>;

  /** Selects node `id` and shows its detail, with the command hidden (inventario §5.1). */
  select: (id: string) => void;
  /** No node selected (the selected one was deleted). */
  clearSelection: () => void;
  showTab: (id: string) => void;
  openValidation: () => void;
  /** Forgets tab `id`; the detail comes in front if it was. */
  tabClosed: (id: string) => void;
  /** "Riprova" put session `next` in the place of `previous` (DEC-033.3). */
  tabReplaced: (previous: string, next: string) => void;
  toggleCommand: () => void;
  /** "Mostra comando" of the context menu: the detail of `id` with its command. */
  showCommand: (id: string) => void;
  setQuery: (query: string) => void;
  setFilter: (filter: TreeFilter) => void;
  setBanner: (banner: DocumentBanner | null) => void;
  requestSearch: () => void;
  setConnectionMenu: (open: boolean) => void;
  setExpanded: (expanded: readonly string[]) => void;
  /**
   * Selects node `id`, opening the folders that hold it (`ancestors`): "Vai al
   * nodo", a copy just made, a node just created.
   */
  reveal: (id: string, ancestors: readonly string[]) => void;
  /** Remembers the summary of the connection of session `id`. */
  sessionOpened: (id: string, summary: string) => void;
  /** Another document is open: nothing of the previous one stays selected. */
  documentSwitched: () => void;
}

const initial = {
  selectedId: null,
  activeTab: DETAIL_TAB,
  validationOpen: false,
  commandShown: false,
  query: "",
  filter: "all",
  banner: null,
  searchRequests: 0,
  connectionMenuOpen: false,
  expanded: null,
  revealRequests: 0,
  sessionSummaries: {},
} satisfies Partial<WorkspaceState>;

/** The store of the main window. */
export const useWorkspace = create<WorkspaceState>()((set) => ({
  ...initial,
  select: (id) => set({ selectedId: id, activeTab: DETAIL_TAB, commandShown: false }),
  clearSelection: () => set({ selectedId: null, commandShown: false }),
  showTab: (id) => set({ activeTab: id }),
  openValidation: () => set({ validationOpen: true, activeTab: VALIDATION_TAB }),
  tabClosed: (id) =>
    set((state) => ({
      validationOpen: id === VALIDATION_TAB ? false : state.validationOpen,
      activeTab: state.activeTab === id ? DETAIL_TAB : state.activeTab,
    })),
  tabReplaced: (previous, next) =>
    set((state) => ({
      activeTab: state.activeTab === previous ? next : state.activeTab,
      sessionSummaries: {
        ...state.sessionSummaries,
        [next]: state.sessionSummaries[previous] ?? "",
      },
    })),
  toggleCommand: () => set((state) => ({ commandShown: !state.commandShown })),
  showCommand: (id) => set({ selectedId: id, activeTab: DETAIL_TAB, commandShown: true }),
  setQuery: (query) => set({ query }),
  setFilter: (filter) => set({ filter }),
  setBanner: (banner) => set({ banner }),
  requestSearch: () => set((state) => ({ searchRequests: state.searchRequests + 1 })),
  setConnectionMenu: (open) => set({ connectionMenuOpen: open }),
  setExpanded: (expanded) => set({ expanded }),
  reveal: (id, ancestors) =>
    set((state) => ({
      selectedId: id,
      activeTab: DETAIL_TAB,
      commandShown: false,
      expanded: [...new Set([...(state.expanded ?? []), ...ancestors])],
      revealRequests: state.revealRequests + 1,
    })),
  sessionOpened: (id, summary) =>
    set((state) => ({ sessionSummaries: { ...state.sessionSummaries, [id]: summary } })),
  documentSwitched: () =>
    set({
      selectedId: null,
      activeTab: DETAIL_TAB,
      validationOpen: false,
      commandShown: false,
      query: "",
      filter: "all",
      banner: null,
      expanded: null,
    }),
}));
