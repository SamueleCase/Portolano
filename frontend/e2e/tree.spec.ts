import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row, rowNames, shoot } from "./mainWindowPage";

/** The rows of the sample with the folders at the root open. */
const SAMPLE = ["Clienti", "ACME", "Admin ACME", "ts01", "Rete", "bastion", "deploy", "vnc-legacy"];

test.describe("the tree", () => {
  test("finds the nodes by name, host or tag and opens their folders", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await page.getByRole("searchbox", { name: texts.tree.search }).fill("web");

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "web01"]);
    await shoot(page, "tree-search");
  });

  test("says when the search finds nothing", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await page.getByRole("searchbox", { name: texts.tree.search }).fill("nessun-nodo-così");

    await expect(page.getByText(texts.tree.noResults)).toBeVisible();
  });

  test("filters the RDP connections", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await page.getByRole("button", { name: texts.tree.filters.rdp, exact: true }).click();

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "dc01", "kiosk", "ts01"]);
  });

  test("filters the favorites", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await page.getByRole("button", { name: texts.tree.filters.favoritesLabel }).click();

    await expect.poll(() => rowNames(tree)).toEqual(["Clienti", "ACME", "web01"]);
  });

  test("moves with the arrows, opens a folder and shows the detail with Enter", async ({
    page,
  }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "Clienti").focus();

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");

    await expect(row(tree, "web01")).toHaveAttribute("aria-selected", "true");
    await expect(page.getByRole("tab", { name: /web01.*dettaglio/ })).toBeVisible();
  });

  test("goes to the parent with ← and closes it with ← again", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "bastion").focus();

    await page.keyboard.press("ArrowLeft");
    await expect(row(tree, "Rete")).toBeFocused();
    await page.keyboard.press("ArrowLeft");

    await expect(row(tree, "bastion")).toHaveCount(0);
  });

  test("jumps to the first and the last row with Home and End", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "deploy").focus();

    await page.keyboard.press("Home");
    await expect(row(tree, "Clienti")).toBeFocused();
    await page.keyboard.press("End");

    await expect(row(tree, "vnc-legacy")).toBeFocused();
  });

  test("opens the context menu with Maiusc+F10 and with the menu key", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "deploy").focus();

    await page.keyboard.press("Shift+F10");
    await expect(page.getByRole("menu", { name: "deploy" })).toBeVisible();
    await page.keyboard.press("Escape");
    await page.keyboard.press("ContextMenu");

    await expect(page.getByRole("menu", { name: "deploy" })).toBeVisible();
  });

  test("jumps to a name typed on the keyboard", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "Clienti").focus();

    await page.keyboard.type("de");

    await expect(row(tree, "deploy")).toBeFocused();
  });

  test("connects to the focused connection with ⌘↩", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "bastion").focus();

    await page.keyboard.press("Meta+Enter");

    await expect(page.getByRole("tab", { name: /bastion.*SSH/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("opens and closes a folder with a double click, without connecting", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await row(tree, "ACME").dblclick();

    await expect(page.getByRole("tab", { name: /SSH|RDP/ })).toHaveCount(0);
    await expect(row(tree, "ACME")).toHaveAttribute("aria-expanded", "false");
  });

  test("reorders a node by dragging it above another", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await row(tree, "deploy").dragTo(row(tree, "bastion"), { targetPosition: { x: 40, y: 2 } });

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

  test("moves a node into a folder by dropping it on the folder", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await row(tree, "vnc-legacy").dragTo(row(tree, "Rete"));

    await expect
      .poll(() => rowNames(tree))
      .toEqual([
        "Clienti",
        "ACME",
        "Admin ACME",
        "ts01",
        "Rete",
        "bastion",
        "deploy",
        "vnc-legacy",
      ]);
    await expect(row(tree, "vnc-legacy")).toHaveAttribute("aria-level", "2");
  });

  test("moves a node one place up with Alt+↑", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "deploy").focus();

    await page.keyboard.press("Alt+ArrowUp");

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
    await expect(row(tree, "deploy")).toBeFocused();
  });

  test("moves a node one place down with «Sposta giù» of its menu", async ({ page }) => {
    await openApp(page);
    const tree = await documentTree(page);

    await row(tree, "bastion").click({ button: "right" });
    await page.getByRole("menuitem", { name: texts.tree.menu.moveDown }).click();

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
    await openApp(page);
    const tree = await documentTree(page);

    await row(tree, "ts01").click({ button: "right" });
    await expect(page.getByRole("menu", { name: "ts01" })).toBeVisible();
    await shoot(page, "context-menu");
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

  test("keeps a row that takes the Tab key after the focused one is filtered away", async ({
    page,
  }) => {
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "deploy").focus();

    await page.getByRole("button", { name: texts.tree.filters.rdp, exact: true }).click();

    await expect(tree.locator('[role="treeitem"][tabindex="0"]')).toHaveCount(1);
  });

  test("starts with the folders at the root open", async ({ page }) => {
    await openApp(page);

    await expect.poll(async () => rowNames(await documentTree(page))).toEqual(SAMPLE);
  });
});
