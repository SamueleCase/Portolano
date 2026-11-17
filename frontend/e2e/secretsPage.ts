import type { Page } from "@playwright/test";

import { expect } from "./fixtures";

/**
 * Everything of the page where a typed secret could stay or go back: the
 * record of the fake backend (every answer, error and event), the DOM, the
 * values of the fields, the storage, the address, the console and the cache
 * of TanStack Query (queries and mutations). SPEC-UI §1.2 and §7.
 */
export interface PageSurfaces {
  traffic: string;
  dom: string;
  fields: string;
  storage: string;
  url: string;
  console: string;
  queryCache: string;
}

/**
 * The cache of TanStack Query of the page, as text. The app does not expose
 * its client: it is found as the `client` prop of `QueryClientProvider` in
 * the tree of React, from the root element. Only the shapes read here are
 * typed: the internals of React are not part of its API.
 */
function readQueryCache(): string {
  interface CachedQuery {
    queryKey: unknown;
    state: { data: unknown; error: unknown };
  }
  interface CachedMutation {
    options: { mutationKey?: unknown };
    state: { variables: unknown; data: unknown; error: unknown; context: unknown };
  }
  interface QueryClient {
    getQueryCache: () => { getAll: () => CachedQuery[] };
    getMutationCache: () => { getAll: () => CachedMutation[] };
  }
  interface Fiber {
    child: Fiber | null;
    sibling: Fiber | null;
    memoizedProps: { client?: QueryClient } | null;
    stateNode: { current?: Fiber } | null;
  }
  const root = document.getElementById("root");
  const key = root && Object.keys(root).find((name) => name.startsWith("__reactContainer$"));
  if (!root || !key) throw new Error("radice di React non trovata");
  const container = Reflect.get(root, key) as Fiber;
  const pending: Fiber[] = [container.stateNode?.current ?? container];
  let client: QueryClient | undefined;
  for (let fiber = pending.pop(); fiber && !client; fiber = pending.pop()) {
    if (typeof fiber.memoizedProps?.client?.getQueryCache === "function") {
      client = fiber.memoizedProps.client;
    }
    if (fiber.sibling) pending.push(fiber.sibling);
    if (fiber.child) pending.push(fiber.child);
  }
  if (!client) throw new Error("QueryClient non trovato");
  const queries = client
    .getQueryCache()
    .getAll()
    .map((query) => [query.queryKey, query.state.data, query.state.error]);
  const mutations = client
    .getMutationCache()
    .getAll()
    .map((mutation) => [
      mutation.options.mutationKey,
      mutation.state.variables,
      mutation.state.data,
      mutation.state.error,
      mutation.state.context,
    ]);
  // An error keeps its message and its own fields (an `IpcFailure` its `error`).
  return JSON.stringify({ queries, mutations }, (_key, value: unknown) =>
    value instanceof Error ? { ...value, name: value.name, message: value.message } : value,
  );
}

/**
 * Watches a page for a typed secret: the console from the first message on,
 * so it starts before the page opens.
 */
export class SecretWatch {
  private readonly messages: string[] = [];

  constructor(private readonly page: Page) {
    page.on("console", (message) => this.messages.push(message.text()));
  }

  /** What the page holds and said, now. */
  async surfaces(): Promise<PageSurfaces> {
    const page = this.page;
    const [traffic, fields, storage, queryCache] = await Promise.all([
      page.evaluate(() => (window.__portolanoFakeTraffic ?? []).join("\n")),
      page
        .locator("input, textarea")
        .evaluateAll((elements) =>
          elements
            .map((element) =>
              element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement
                ? element.value
                : "",
            )
            .join("\n"),
        ),
      page.evaluate(() => JSON.stringify([{ ...localStorage }, { ...sessionStorage }])),
      page.evaluate(readQueryCache),
    ]);
    return {
      traffic,
      dom: await page.content(),
      fields,
      storage,
      url: page.url(),
      console: this.messages.join("\n"),
      queryCache,
    };
  }

  /** `secret` is on none of the surfaces of the page. */
  async expectNowhere(secret: string): Promise<void> {
    const surfaces = await this.surfaces();
    for (const [surface, text] of Object.entries(surfaces)) {
      expect(text, `il segreto compare in: ${surface}`).not.toContain(secret);
    }
  }
}
