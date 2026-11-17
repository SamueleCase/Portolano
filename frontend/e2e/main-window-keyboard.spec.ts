import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import {
  BACK_TAB,
  type FocusStop,
  TAB,
  focusedStop,
  sweep,
  tabTo,
  withoutTransitions,
} from "./keyboardPage";
import { documentTree, openApp, row } from "./mainWindowPage";

/**
 * The main window from the keyboard (SPEC-UI §1.6, §5.5): Tab reaches every
 * area with its focus visible, the toolbar and the tree as one stop each;
 * a dialog and the side panel of the editor keep the focus inside and give
 * it back to the control that opened them.
 */

/** The names of the stops in `area`. */
function namesIn(stops: FocusStop[], area: string): string[] {
  return stops.filter((stop) => stop.area === area).map((stop) => stop.name);
}

/** The window with the detail of bastion and the focus on its row. */
async function windowOnBastion(page: Page): Promise<void> {
  await openApp(page);
  await row(await documentTree(page), "bastion").click();
  await expect(page.getByRole("heading", { name: "bastion" })).toBeVisible();
  await withoutTransitions(page);
}

/**
 * Presses Tab, then Shift+Tab, twice as many times as `surface` has stops,
 * and returns whether the focus ever left it.
 */
async function focusLeaves(page: Page, surface: Locator): Promise<boolean> {
  const stops = await sweep(page);
  for (const key of [TAB, BACK_TAB]) {
    for (let step = 0; step < stops.length * 2; step++) {
      await page.keyboard.press(key);
      const inside = await surface.evaluate((element) => element.contains(document.activeElement));
      if (!inside) return true;
    }
  }
  return false;
}

test.describe("the main window from the keyboard", () => {
  test("reaches every area with Tab, the toolbar and the tree as one stop each", async ({
    page,
  }) => {
    await windowOnBastion(page);

    const stops = await sweep(page);

    const { filters } = texts.tree;
    expect(namesIn(stops, "toolbar")).toHaveLength(1);
    expect(namesIn(stops, "search")).toHaveLength(1);
    expect(namesIn(stops, "filters")).toEqual([
      filters.all,
      filters.ssh,
      filters.rdp,
      filters.credentials,
      filters.favoritesLabel,
    ]);
    expect(namesIn(stops, "tree")).toHaveLength(1);
    expect(namesIn(stops, "actions")).toEqual(
      expect.arrayContaining([texts.details.actions.connect, texts.details.actions.edit]),
    );
    expect(namesIn(stops, "footer")).toEqual([texts.components.vault.fileLocked]);
  });

  test("shows the focus at every stop of Tab", async ({ page }) => {
    await windowOnBastion(page);

    const stops = await sweep(page);

    expect(stops.filter((stop) => !stop.visible).map((stop) => stop.name)).toEqual([]);
  });

  test("moves among every control of the toolbar with the arrows, showing the focus", async ({
    page,
  }) => {
    await windowOnBastion(page);
    const toolbar = page.getByRole("toolbar", { name: texts.app.toolbar.label });
    const items = await toolbar.getByRole("button").count();
    await tabTo(page, "toolbar");

    const stops: FocusStop[] = [];
    for (let step = 0; step < items; step++) {
      const stop = await focusedStop(page);
      if (stop) stops.push(stop);
      await page.keyboard.press("ArrowRight");
    }

    expect(new Set(stops.map((stop) => stop.id)).size).toBe(items);
    const unseen = stops.filter((stop) => stop.area !== "toolbar" || !stop.visible);
    expect(unseen, "controlli della barra senza fuoco visibile").toEqual([]);
  });

  test("keeps the focus in a dialog and gives it back to its button on Esc", async ({ page }) => {
    await windowOnBastion(page);
    const preferences = page.getByRole("button", { name: texts.app.toolbar.preferences });
    await preferences.focus();
    await page.keyboard.press("Enter");
    const dialog = page.getByRole("dialog", { name: texts.preferences.title });
    await expect(dialog).toBeVisible();

    expect(await focusLeaves(page, dialog)).toBe(false);
    await page.keyboard.press("Escape");

    await expect(dialog).toBeHidden();
    await expect(preferences).toBeFocused();
  });

  test("keeps the focus in the editor and gives it back to Modifica on Esc", async ({ page }) => {
    await windowOnBastion(page);
    const edit = page.getByRole("button", { name: texts.details.actions.edit, exact: true });
    await edit.focus();
    await page.keyboard.press("Enter");
    const panel = page.getByRole("dialog", { name: /bastion/ });
    await expect(panel).toBeVisible();

    expect(await focusLeaves(page, panel)).toBe(false);
    await page.keyboard.press("Escape");

    await expect(panel).toBeHidden();
    await expect(edit).toBeFocused();
  });
});
