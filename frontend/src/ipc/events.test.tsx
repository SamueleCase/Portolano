import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { emit } from "@tauri-apps/api/event";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { installFakeBackend } from "../test/fakeBackend";
import { useBackendEvents } from "./events";
import { queryKeys } from "./keys";

let unmount: () => void = () => undefined;

// The listeners stop asynchronously: they must be gone before the shared
// teardown removes the mocked event plugin.
afterEach(async () => {
  unmount();
  await new Promise((resolve) => setTimeout(resolve, 0));
});

function setup(handlers: Parameters<typeof useBackendEvents>[0] = {}) {
  installFakeBackend();
  const client = new QueryClient();
  client.setQueryData(queryKeys.tree, { name: "vecchio", nodes: [] });
  client.setQueryData(queryKeys.vault, { kind: "file", path: null, exists: true, locked: false });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  unmount = renderHook(() => useBackendEvents(handlers), { wrapper }).unmount;
  return client;
}

describe("useBackendEvents", () => {
  it("makes the document stale when the app saved it", async () => {
    const client = setup();

    await act(() => emit("document-changed", { fileName: "portolano.json" }));

    await waitFor(() => expect(client.getQueryState(queryKeys.tree)?.isInvalidated).toBe(true));
  });

  it("makes the vault and the secrets stale when the vault is locked", async () => {
    const client = setup();

    await act(() => emit("vault-locked", { kind: "file" }));

    await waitFor(() => expect(client.getQueryState(queryKeys.vault)?.isInvalidated).toBe(true));
    expect(client.getQueryState(queryKeys.tree)?.isInvalidated).toBe(true);
  });

  it("tells the interface that another program changed the document", async () => {
    const onModifiedExternally = vi.fn();
    setup({ onModifiedExternally });

    await act(() => emit("document-modified-externally", { fileName: "portolano.json" }));

    await waitFor(() => expect(onModifiedExternally).toHaveBeenCalledWith("portolano.json"));
  });

  it("passes the command of a menu item", async () => {
    const onMenu = vi.fn();
    setup({ onMenu });

    await act(() => emit("menu-action", { command: { kind: "duplicate" } }));

    await waitFor(() => expect(onMenu).toHaveBeenCalledWith({ kind: "duplicate" }));
  });

  it("tells the interface that the document is gone and refreshes the documents", async () => {
    const onMissing = vi.fn();
    const client = setup({ onMissing });
    client.setQueryData(queryKeys.recentDocuments, []);

    await act(() => emit("document-missing", { fileName: "portolano.json" }));

    await waitFor(() => expect(onMissing).toHaveBeenCalledWith("portolano.json"));
    expect(client.getQueryState(queryKeys.recentDocuments)?.isInvalidated).toBe(true);
  });

  it("stops listening when unmounted", async () => {
    const onMenu = vi.fn();
    setup({ onMenu });
    await act(() => emit("menu-action", { command: { kind: "search" } }));
    await waitFor(() => expect(onMenu).toHaveBeenCalledTimes(1));

    unmount();
    unmount = () => undefined;
    await new Promise((resolve) => setTimeout(resolve, 0));
    await act(() => emit("menu-action", { command: { kind: "search" } }));

    expect(onMenu).toHaveBeenCalledTimes(1);
  });
});
