import type { Page } from "@playwright/test";

import type { MenuCommand } from "../src/bindings";
import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import {
  addButton,
  addFromSidebar,
  documentTree,
  emitEvent,
  openApp,
  row,
  shoot,
  sidebarTool,
} from "./mainWindowPage";

/** The bar of the window. */
function windowBar(page: Page) {
  return page.getByRole("banner", { name: texts.app.toolbar.label });
}

/** The editor of a new node of `kind`, open. */
function newNodeEditor(kind: "folder" | "credential") {
  return (page: Page) =>
    expect(page.getByRole("dialog", { name: texts.editor.subtitles.add[kind] })).toBeVisible();
}

/** The import wizard, open. */
function importWizard(page: Page) {
  return expect(page.getByRole("dialog", { name: texts.importWizard.title })).toBeVisible();
}

/** What each entry of "+" opens. */
const ADD_ENTRIES: [string, (page: Page) => Promise<void>][] = [
  [texts.tree.add.newFolder, newNodeEditor("folder")],
  [texts.tree.add.newCredential, newNodeEditor("credential")],
  [texts.tree.add.import, importWizard],
];

/** What each command of the native menu whose button left the bar opens (DEC-050.4). */
const MENU_COMMANDS: [MenuCommand["kind"], (page: Page) => Promise<void>][] = [
  [
    "newConnection",
    (page) =>
      expect(page.getByRole("menu", { name: texts.app.toolbar.newConnectionMenu })).toBeVisible(),
  ],
  ["newFolder", newNodeEditor("folder")],
  ["newCredential", newNodeEditor("credential")],
  ["import", importWizard],
  [
    "validate",
    (page) =>
      expect(page.getByRole("tab", { name: /Validazione/ })).toHaveAttribute(
        "aria-selected",
        "true",
      ),
  ],
  [
    "preferences",
    (page) => expect(page.getByRole("dialog", { name: texts.preferences.title })).toBeVisible(),
  ],
];

test.describe("the main window", () => {
  test("shows the bar, the tree and the vault of the document", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await expect(page.getByText("portolano.json")).toBeVisible();
    await expect(windowBar(page).getByRole("button")).toHaveText([texts.app.toolbar.connect]);
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

test.describe("the «+» of the sidebar", () => {
  test("offers the new nodes and the import", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await addButton(page).click();

    await expect(
      page.getByRole("menu", { name: texts.tree.add.label }).getByRole("menuitem"),
    ).toHaveText([
      new RegExp(`^${texts.tree.add.newConnection}`),
      new RegExp(`^${texts.tree.add.newFolder}`),
      new RegExp(`^${texts.tree.add.newCredential}`),
      new RegExp(`^${texts.tree.add.import}`),
    ]);
    await shoot(page, "add-menu");
  });

  test("asks the protocol of a new connection and opens its editor", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await addFromSidebar(page, texts.tree.add.newConnection);
    const protocols = page.getByRole("menu", { name: texts.app.toolbar.newConnectionMenu });
    await expect(protocols).toBeVisible();
    await shoot(page, "add-menu-protocol");
    await protocols.getByRole("menuitem", { name: texts.app.toolbar.protocols.ssh }).click();

    await expect(page.getByRole("dialog", { name: texts.editor.subtitles.add.ssh })).toBeVisible();
  });

  for (const [entry, opened] of ADD_ENTRIES) {
    test(`opens its function from «${entry}»`, async ({ page }) => {
      await openApp(page);
      await documentTree(page);

      await addFromSidebar(page, entry);

      await opened(page);
    });
  }
});

test.describe("the tools of the footer of the sidebar", () => {
  test("opens the validation tab with Valida", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await sidebarTool(page, "validate").click();

    await expect(page.getByRole("tab", { name: /Validazione/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("opens the preferences with Preferenze", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await sidebarTool(page, "preferences").click();

    await expect(page.getByRole("dialog", { name: texts.preferences.title })).toBeVisible();
  });

  test("names each tool in its tooltip, with the shortcut", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await sidebarTool(page, "preferences").hover();

    await expect(page.getByRole("tooltip")).toHaveText(
      `${texts.tree.tools.preferences}${texts.app.shortcuts.preferences}`,
    );
    await shoot(page, "footer-tools");
  });
});

test.describe("the native menu", () => {
  for (const [kind, opened] of MENU_COMMANDS) {
    test(`still performs «${kind}», whose button left the bar`, async ({ page }) => {
      await openApp(page);
      await documentTree(page);

      await emitEvent(page, "menu-action", { command: { kind } });

      await opened(page);
    });
  }
});
