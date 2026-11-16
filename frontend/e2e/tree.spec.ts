import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";

const SHOTS = "e2e-screenshots/main-window";

async function openApp(page: Page): Promise<Locator> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  const tree = page.getByRole("tree", { name: texts.tree.label });
  await expect(tree).toBeVisible();
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
  return tree;
}

/** The names of the rows shown, in order. */
async function rowNames(tree: Locator): Promise<string[]> {
  return tree
    .getByRole("treeitem")
    .evaluateAll((rows) => rows.map((row) => row.getAttribute("aria-label") ?? ""));
}

test.describe("the tree", () => {
  test("finds the nodes by name, host or tag and opens their folders", async ({ page }) => {
    const tree = await openApp(page);

    await page.getByRole("searchbox", { name: texts.tree.search }).fill("web");

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "web01"]);
    await page.screenshot({ path: `${SHOTS}/tree-search.png`, animations: "disabled" });
  });

  test("says when the search finds nothing", async ({ page }) => {
    await openApp(page);

    await page.getByRole("searchbox", { name: texts.tree.search }).fill("nessun-nodo-così");

    await expect(page.getByText(texts.tree.noResults)).toBeVisible();
  });

  test("filters the RDP connections", async ({ page }) => {
    const tree = await openApp(page);

    await page.getByRole("button", { name: texts.tree.filters.rdp, exact: true }).click();

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "dc01", "kiosk", "ts01"]);
  });

  test("filters the favorites", async ({ page }) => {
    const tree = await openApp(page);

    await page.getByRole("button", { name: texts.tree.filters.favoritesLabel }).click();

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "web01"]);
  });

  test("moves with the arrows, opens a folder and shows the detail with Enter", async ({
    page,
  }) => {
    const tree = await openApp(page);
    await tree.getByRole("treeitem", { name: "Clienti" }).focus();

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    await expect(tree.getByRole("treeitem", { name: "web01" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(page.getByRole("tab", { name: /web01.*dettaglio/ })).toBeVisible();
  });

  test("jumps to a name typed on the keyboard", async ({ page }) => {
    const tree = await openApp(page);
    await tree.getByRole("treeitem", { name: "Clienti" }).focus();

    await page.keyboard.type("de");

    await expect(tree.getByRole("treeitem", { name: "deploy" })).toBeFocused();
  });

  test("connects to the focused connection with ⌘↩", async ({ page }) => {
    const tree = await openApp(page);
    await tree.getByRole("treeitem", { name: "bastion" }).focus();

    await page.keyboard.press("Meta+Enter");

    await expect(page.getByRole("tab", { name: /bastion.*SSH/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("reorders a node by dragging it above another", async ({ page }) => {
    const tree = await openApp(page);
    const bastion = tree.getByRole("treeitem", { name: "bastion" });

    await tree.getByRole("treeitem", { name: "deploy" }).dragTo(bastion, {
      targetPosition: { x: 40, y: 2 },
    });

    await expect
      .poll(() => rowNames(tree))
      .toEqual([
        "Clienti",
        "ACME",
        "Admin ACME",
        "ts01",
        "Rete",
        "deploy",
        "bastion",
        "vnc-legacy",
      ]);
  });

  test("moves a node into a folder with «Sposta in…» of its menu", async ({ page }) => {
    const tree = await openApp(page);

    await tree.getByRole("treeitem", { name: "ts01" }).click({ button: "right" });
    await expect(page.getByRole("menu", { name: "ts01" })).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/context-menu.png`, animations: "disabled" });
    await page.getByRole("menuitem", { name: texts.tree.menu.moveTo }).click();
    const picker = page.getByRole("dialog");
    await picker.getByRole("option", { name: /Rete/ }).click();
    await picker.getByRole("button", { name: /Scegli/ }).click();

    await expect
      .poll(() => rowNames(tree))
      .toEqual([
        "Clienti",
        "ACME",
        "Admin ACME",
        "Rete",
        "bastion",
        "deploy",
        "ts01",
        "vnc-legacy",
      ]);
  });
});
