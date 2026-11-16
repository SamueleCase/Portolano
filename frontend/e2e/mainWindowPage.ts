import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import type { FakeSeed } from "../src/test/fakeBackend";
import { expect } from "./fixtures";

/** Where the screenshots of the main window go (checkpoint 2): outside `src/`, ignored by git. */
export const SHOTS = "e2e-screenshots/main-window";

/**
 * Opens the app over the fake backend started as `seed` says (the sample
 * document by default), with still motion and the fonts loaded.
 */
export async function openApp(page: Page, seed: FakeSeed = {}): Promise<void> {
  await page.addInitScript((value) => {
    window.__portolanoFakeSeed = value;
  }, seed);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

/** The tree of the document, once shown. */
export async function documentTree(page: Page): Promise<Locator> {
  const tree = page.getByRole("tree", { name: texts.tree.label });
  await expect(tree).toBeVisible();
  return tree;
}

function escaped(text: string): string {
  return text.replaceAll(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** The row of node `name`: its accessible name starts with the name and a comma. */
export function row(scope: Page | Locator, name: string): Locator {
  return scope.getByRole("treeitem", { name: new RegExp(`^${escaped(name)},`) });
}

/** The names of the rows shown, in order. */
export async function rowNames(tree: Locator): Promise<string[]> {
  return tree
    .getByRole("treeitem")
    .evaluateAll((rows) =>
      rows.map((element) => (element.getAttribute("aria-label") ?? "").split(", ")[0] ?? ""),
    );
}

/** An event of the backend, through the event plugin of the fake IPC. */
export async function emitEvent(page: Page, event: string, payload: unknown): Promise<void> {
  await page.evaluate(
    async ([name, data]) => {
      // The fake IPC of `mockIPC` lives on this global, which has no types.
      const internals = Reflect.get(window, "__TAURI_INTERNALS__") as {
        invoke: (command: string, args: unknown) => Promise<unknown>;
      };
      await internals.invoke("plugin:event|emit", { event: name, payload: data });
    },
    [event, payload] as const,
  );
}

/** A screenshot of the whole window for the visual checkpoint. */
export async function shoot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}
