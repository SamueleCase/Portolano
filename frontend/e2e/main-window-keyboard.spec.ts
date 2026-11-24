import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import {
  BACK_TAB,
  type FocusStop,
  TAB,
  press,
  markControls,
  missedControls,
  sweep,
  tabTo,
  withoutTransitions,
} from "./keyboardPage";
import { addButton, documentTree, openApp, row, sidebarTool } from "./mainWindowPage";

/**
 * The main window from the keyboard (SPEC-UI §1.6, §5.5, DEC-050): Tab
 * reaches every area with its focus visible, the tree as one stop; the menu
 * of "+" moves with the arrows and gives the focus back on Esc; a dialog and
 * the side panel of the editor keep the focus inside and give it back to the
 * control that opened them.
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
 * and returns whether the focus ever left it. A surface of one stop would
 * keep the focus without a trap: it must have more.
 */
async function focusLeaves(page: Page, surface: Locator): Promise<boolean> {
  const stops = await sweep(page);
  expect(stops.length, "fermate di Tab nella superficie").toBeGreaterThan(1);
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
  test("reaches every control of the window with Tab", async ({ page }) => {
    await windowOnBastion(page);
    const total = await markControls(page);

    const stops = await sweep(page, TAB, total + 20);

    expect(await missedControls(page, total, stops), "controlli non raggiungibili con Tab").toEqual(
      [],
    );
  });

  test("reaches every area with Tab, the tree as one stop", async ({ page }) => {
    await windowOnBastion(page);

    const stops = await sweep(page);

    const { filters, add, tools } = texts.tree;
    expect(namesIn(stops, "bar")).toEqual([texts.app.toolbar.connect]);
    expect(namesIn(stops, "search")).toHaveLength(1);
    expect(namesIn(stops, "add")).toEqual([add.label]);
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
    expect(namesIn(stops, "tools")).toEqual([tools.validate, tools.preferences]);
  });

  test("shows the focus at every stop of Tab", async ({ page }) => {
    await windowOnBastion(page);

    const stops = await sweep(page);

    expect(stops.filter((stop) => !stop.visible).map((stop) => stop.name)).toEqual([]);
  });

  test("opens the menu of «+» with Enter and moves among its entries with the arrows", async ({
    page,
  }) => {
    await windowOnBastion(page);
    await tabTo(page, "add");
    const menu = page.getByRole("menu", { name: texts.tree.add.label });

    const stops = [await press(page, "Enter")];
    const entries = await menu.getByRole("menuitem").count();
    for (let step = 1; step < entries; step++) stops.push(await press(page, "ArrowDown"));

    expect(new Set(stops.map((stop) => stop?.id)).size).toBe(entries);
    const unseen = stops.filter((stop) => stop?.area !== "menu" || !stop.visible);
    expect(unseen, "voci del menu senza fuoco visibile").toEqual([]);
  });

  test("closes the menu of «+» with Esc and gives the focus back to «+»", async ({ page }) => {
    await windowOnBastion(page);
    await tabTo(page, "add");
    await page.keyboard.press("Space");
    const menu = page.getByRole("menu", { name: texts.tree.add.label });
    await expect(menu).toBeVisible();

    await page.keyboard.press("Escape");

    await expect(menu).toBeHidden();
    await expect(addButton(page)).toBeFocused();
  });

  test("reaches the tools of the footer with Tab, showing the focus", async ({ page }) => {
    await windowOnBastion(page);

    const first = await tabTo(page, "tools");
    const second = await press(page, TAB);

    expect([first, second].map((stop) => [stop?.name, stop?.visible])).toEqual([
      [texts.tree.tools.validate, true],
      [texts.tree.tools.preferences, true],
    ]);
  });

  test("keeps the focus in a dialog and gives it back to its button on Esc", async ({ page }) => {
    await windowOnBastion(page);
    const preferences = sidebarTool(page, "preferences");
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
