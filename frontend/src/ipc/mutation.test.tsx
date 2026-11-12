import { type QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";

import { TEST_MASTER_PASSWORD, installFakeBackend } from "../test/fakeBackend";
import { unlocked } from "../test/fakeBackendFixtures";
import { useAddNode, useReloadDocument, useUnlockVault } from "./mutations";
import { createQueryClient } from "./queryClient";
import { treeQuery, vaultStatusQuery } from "./queries";

/** A sentinel secret: it must never be found in the cache of Query. */
const SENTINEL = "sentinel-frontend-secret-91c2";

function wrapperWith(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

function newClient(): QueryClient {
  return createQueryClient();
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

/** A credential at the root named `name`, with the sentinel as password. */
function credentialWithSentinel(name: string) {
  return {
    parent: null,
    name,
    kind: { type: "credential" as const, username: "operatore", domain: null, keyFile: null },
    details: {},
    secrets: { password: SENTINEL },
  };
}

describe("secret mutations", () => {
  it("keep the master password out of the cache of Query", async () => {
    installFakeBackend();
    const client = newClient();
    const { result } = renderHook(() => useUnlockVault(), { wrapper: wrapperWith(client) });

    await act(() => result.current.mutateAsync(TEST_MASTER_PASSWORD));

    expect(cacheText(client)).not.toContain(TEST_MASTER_PASSWORD);
    expect(typeof client.getMutationCache().getAll()[0]?.state.variables).toBe("number");
  });

  it("keep a typed password out of the cache of Query", async () => {
    unlocked();
    const client = newClient();
    const { result } = renderHook(() => useAddNode(), { wrapper: wrapperWith(client) });

    await act(() => result.current.mutateAsync(credentialWithSentinel("Nuova")));

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(cacheText(client)).not.toContain(SENTINEL);
    expect(result.current.holdsArguments()).toBe(false);
  });

  it("keep a refused password out of the cache and let it go", async () => {
    installFakeBackend();
    const client = newClient();
    const { result } = renderHook(() => useUnlockVault(), { wrapper: wrapperWith(client) });

    await act(async () => {
      await expect(result.current.mutateAsync(SENTINEL)).rejects.toThrow();
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(cacheText(client)).not.toContain(SENTINEL);
    expect(result.current.holdsArguments()).toBe(false);
  });

  it("give each of two calls in a row its own arguments", async () => {
    const state = unlocked();
    const client = newClient();
    const { result } = renderHook(() => useAddNode(), { wrapper: wrapperWith(client) });

    const [first, second] = await act(() =>
      Promise.all([
        result.current.mutateAsync(credentialWithSentinel("Prima")),
        result.current.mutateAsync(credentialWithSentinel("Seconda")),
      ]),
    );

    expect([first.path, second.path]).toEqual(["/Prima", "/Seconda"]);
    expect(state.document?.nodes.map((node) => node.name)).toEqual(
      expect.arrayContaining(["Prima", "Seconda"]),
    );
    expect(result.current.holdsArguments()).toBe(false);
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

  it("run a command without arguments with a bare call", async () => {
    installFakeBackend();
    const { result } = renderHook(() => useReloadDocument(), {
      wrapper: wrapperWith(newClient()),
    });

    const info = await act(() => result.current.mutateAsync());

    expect(info.fileName).toBe("portolano.json");
  });
});
