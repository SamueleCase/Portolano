import { queryOptions } from "@tanstack/react-query";

import { commands } from "../bindings";
import { queryKeys } from "./keys";
import { unwrap } from "./result";

/** The documents, the vault backend and the preferences of the interface. */
export const appStatusQuery = queryOptions({
  queryKey: queryKeys.appStatus,
  queryFn: async () => unwrap(await commands.appStatus()),
});

/** The documents opened last. */
export const recentDocumentsQuery = queryOptions({
  queryKey: queryKeys.recentDocuments,
  queryFn: async () => unwrap(await commands.recentDocuments()),
});

/** The whole document as a tree, with the state of the secrets. */
export const treeQuery = queryOptions({
  queryKey: queryKeys.tree,
  queryFn: async () => unwrap(await commands.tree()),
});

/** The detail of a node. */
export function nodeDetailsQuery(id: string) {
  return queryOptions({
    queryKey: queryKeys.nodeDetails(id),
    queryFn: async () => unwrap(await commands.nodeDetails(id)),
  });
}

/** The secrets of a node and whether the vault has them. */
export function secretStatusQuery(id: string) {
  return queryOptions({
    queryKey: queryKeys.secretStatus(id),
    queryFn: async () => unwrap(await commands.secretStatus(id)),
  });
}

/** The nodes whose name, host or tag contains `query`. */
export function searchQuery(query: string) {
  return queryOptions({
    queryKey: queryKeys.search(query),
    queryFn: async () => unwrap(await commands.search(query)),
  });
}

/** The problems of the document ("Ricontrolla" refetches it). */
export const validationQuery = queryOptions({
  queryKey: queryKeys.validation,
  queryFn: async () => unwrap(await commands.validate()),
});

/** The launch plan of a connection ("Mostra comando"). */
export function connectPreviewQuery(id: string) {
  return queryOptions({
    queryKey: queryKeys.connectPreview(id),
    queryFn: async () => unwrap(await commands.connectPreview(id)),
  });
}

/** The login to ask before connecting, if any. */
export function loginRequestQuery(id: string) {
  return queryOptions({
    queryKey: queryKeys.loginRequest(id),
    queryFn: async () => unwrap(await commands.loginRequest(id)),
  });
}

/** The images stored with the open document. */
export const customIconsQuery = queryOptions({
  queryKey: queryKeys.customIcons,
  queryFn: async () => unwrap(await commands.customIcons()),
});

/** The names of the builtin icons; they never change while the app runs. */
export const builtinIconsQuery = queryOptions({
  queryKey: queryKeys.builtinIcons,
  queryFn: async () => unwrap(await commands.builtinIcons()),
  staleTime: Infinity,
});

/** The vault of the configuration and whether it is locked. */
export const vaultStatusQuery = queryOptions({
  queryKey: queryKeys.vault,
  queryFn: async () => unwrap(await commands.vaultStatus()),
});

/** The keys of `config.toml` and their values. */
export const configQuery = queryOptions({
  queryKey: queryKeys.config,
  queryFn: async () => unwrap(await commands.configGet()),
});

/** The preferences of the interface. */
export const uiPreferencesQuery = queryOptions({
  queryKey: queryKeys.uiPreferences,
  queryFn: async () => unwrap(await commands.uiPrefsGet()),
});
