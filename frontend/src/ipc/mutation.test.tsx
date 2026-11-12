import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";

import { TEST_MASTER_PASSWORD, initialFakeState, installFakeBackend } from "../test/fakeBackend";
import { useAddNode, useUnlockVault } from "./mutations";
import { treeQuery, vaultStatusQuery } from "./queries";

/** A sentinel secret: it must never be found in the cache of Query. */
const SENTINEL = "sentinel-frontend-secret-91c2";

function wrapperWith(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

function newClient(): QueryClient {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

/** Everything the cache of Query holds, as text. */
function cacheText(client: QueryClient): string {
  return JSON.stringify([
    client
      .getMutationCache()
      .getAll()
      .map((mutation) => mutation.state),
    client
      .getQueryCache()
      .getAll()
      .map((query) => query.state),
  ]);
}

describe("secret mutations", () => {
  it("keep the master password out of the cache of Query", async () => {
    installFakeBackend();
    const client = newClient();
    const { result } = renderHook(() => useUnlockVault(), { wrapper: wrapperWith(client) });

    await act(() => result.current.mutateAsync(TEST_MASTER_PASSWORD));

    expect(cacheText(client)).not.toContain(TEST_MASTER_PASSWORD);
    expect(client.getMutationCache().getAll()[0]?.state.variables).toBeUndefined();
  });

  it("keep a typed password out of the cache of Query", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    installFakeBackend(state);
    const client = newClient();
    const { result } = renderHook(() => useAddNode(), { wrapper: wrapperWith(client) });

    await act(() =>
      result.current.mutateAsync({
        parent: null,
        name: "Nuova",
        kind: { type: "credential", username: "operatore", domain: null, keyFile: null },
        details: {},
        secrets: { password: SENTINEL },
      }),
    );

    expect(result.current.isSuccess).toBe(true);
    expect(cacheText(client)).not.toContain(SENTINEL);
  });
});

describe("mutations", () => {
  it("make the data they change stale", async () => {
    installFakeBackend();
    const client = newClient();
    const { result } = renderHook(
      () => ({ vault: useQuery(vaultStatusQuery), unlock: useUnlockVault() }),
      { wrapper: wrapperWith(client) },
    );
    await waitFor(() => expect(result.current.vault.data?.locked).toBe(true));

    await act(() => result.current.unlock.mutateAsync(TEST_MASTER_PASSWORD));

    await waitFor(() => expect(result.current.vault.data?.locked).toBe(false));
  });

  it("refresh the tree after adding a node", async () => {
    installFakeBackend();
    const client = newClient();
    const { result } = renderHook(() => ({ tree: useQuery(treeQuery), add: useAddNode() }), {
      wrapper: wrapperWith(client),
    });
    await waitFor(() => expect(result.current.tree.data).toBeDefined());

    await act(() =>
      result.current.add.mutateAsync({
        parent: null,
        name: "Nuova cartella",
        kind: { type: "folder", credential: { mode: "inherit" } },
        details: {},
        secrets: {},
      }),
    );

    await waitFor(() =>
      expect(result.current.tree.data?.nodes.map((node) => node.name)).toContain("Nuova cartella"),
    );
  });
});
