/**
 * The keys of the backend data in the TanStack Query cache. Everything read
 * from the open document sits under `document`, so that one invalidation
 * refreshes it after a save or an event; the rest is per area.
 */
export const queryKeys = {
  appStatus: ["app-status"] as const,
  recentDocuments: ["recent-documents"] as const,
  document: ["document"] as const,
  tree: ["document", "tree"] as const,
  nodeDetails: (id: string) => ["document", "node", id] as const,
  secretStatus: (id: string) => ["document", "node", id, "secrets"] as const,
  search: (query: string) => ["document", "search", query] as const,
  validation: ["document", "validation"] as const,
  connectPreview: (id: string) => ["document", "connect-preview", id] as const,
  loginRequest: (id: string) => ["document", "login-request", id] as const,
  customIcons: ["document", "custom-icons"] as const,
  vault: ["vault"] as const,
  config: ["config"] as const,
  uiPreferences: ["ui-preferences"] as const,
  builtinIcons: ["builtin-icons"] as const,
};
