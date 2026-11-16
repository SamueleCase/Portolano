import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { documentTree, emitEvent, openApp, row, shoot } from "./mainWindowPage";

test.describe("the main window", () => {
  test("shows the toolbar, the tree and the vault of the document", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await expect(page.getByText("portolano.json")).toBeVisible();
    await expect(page.getByRole("toolbar", { name: texts.app.toolbar.label })).toBeVisible();
    await expect(row(page, "Clienti")).toBeVisible();
    await expect(page.getByText(texts.components.vault.fileLocked)).toBeVisible();
    await shoot(page, "main-window");
  });

  test("shows a skeleton while the tree loads", async ({ page }) => {
    await openApp(page, { unanswered: ["tree"] });

    await expect(page.getByRole("status", { name: texts.tree.loading })).toBeVisible();
    await shoot(page, "tree-skeleton");
  });

  test("uses the fonts shipped with the app", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    const loaded = await page.evaluate(async () => {
      await document.fonts.ready;
      return [...document.fonts]
        .filter((face) => face.status === "loaded")
        .map((face) => face.family.replaceAll(/["']/g, ""));
    });

    expect(loaded).toEqual(expect.arrayContaining(["Hanken Grotesk", "IBM Plex Mono"]));
  });

  test("disables the functions not mounted yet, with the reason", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    const importButton = page.getByRole("button", { name: texts.app.toolbar.import });
    await expect(importButton).toHaveAttribute("aria-disabled", "true");
    await importButton.hover();
    await expect(page.getByRole("tooltip")).toHaveText(texts.app.reasons.unavailable);
  });

  test("connects to an SSH connection in a tab with its terminal", async ({ page }) => {
    // Open vault: bastion's credential has a passphrase in the vault, so with
    // the vault locked the backend refuses and the window asks to unlock (DEC-042).
    await openApp(page, { vaultLocked: false });

    await row(await documentTree(page), "bastion").dblclick();

    const tab = page.getByRole("tab", { name: /bastion.*SSH/ });
    await expect(tab).toHaveAttribute("aria-selected", "true");
    const panel = page.getByRole("tabpanel");
    await expect(panel.getByRole("status").first()).toHaveText(texts.sessions.status.connected);
    await expect(panel).toContainText("Last login");
    await shoot(page, "ssh-session");
  });

  test("brings the open session in front instead of opening another", async ({ page }) => {
    // Open vault: bastion's credential has a passphrase in the vault, so with
    // the vault locked the backend refuses and the window asks to unlock (DEC-042).
    await openApp(page, { vaultLocked: false });
    const tree = await documentTree(page);
    await row(tree, "bastion").dblclick();
    const session = page.getByRole("tab", { name: /bastion.*SSH/ });
    await expect(session).toBeVisible();
    await page.getByRole("tab", { name: /dettaglio/ }).click();

    await row(tree, "bastion").dblclick();

    await expect(session).toHaveCount(1);
    await expect(session).toHaveAttribute("aria-selected", "true");
  });

  test("closes the session tab with ⌘W and goes back to the detail", async ({ page }) => {
    // Open vault: bastion's credential has a passphrase in the vault, so with
    // the vault locked the backend refuses and the window asks to unlock (DEC-042).
    await openApp(page, { vaultLocked: false });
    await row(await documentTree(page), "bastion").dblclick();
    const session = page.getByRole("tab", { name: /bastion.*SSH/ });
    await expect(session).toHaveAttribute("aria-selected", "true");

    await session.focus();
    await page.keyboard.press("Meta+KeyW");

    await expect(page.getByRole("tab", { name: /SSH/ })).toHaveCount(0);
  });

  test("offers to reload a document changed by another program", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await emitEvent(page, "document-modified-externally", { fileName: "portolano.json" });

    const banner = page.getByRole("alert").filter({ hasText: "portolano.json" });
    await expect(banner).toContainText(
      interpolate(texts.app.banner.modified, { file: "portolano.json" }),
    );
    await shoot(page, "modified-banner");
    await banner.getByRole("button", { name: texts.app.banner.reload }).click();
    await expect(banner).toHaveCount(0);
  });

  test("writes the document back where it was with «Ricrea»", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await emitEvent(page, "document-missing", { fileName: "portolano.json" });

    const banner = page.getByRole("alert").filter({ hasText: "portolano.json" });
    await expect(banner.getByRole("button", { name: texts.app.banner.openOther })).toBeVisible();
    await shoot(page, "missing-banner");
    await banner.getByRole("button", { name: texts.app.banner.recreate }).click();
    await expect(banner).toHaveCount(0);
    await expect(row(page, "Clienti")).toBeVisible();
  });

  test("performs the items of the native menu", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await emitEvent(page, "menu-action", { command: { kind: "densityCompact" } });

    await expect(page.locator("html")).toHaveAttribute("data-density", "compact");
  });
});
