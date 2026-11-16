import type { Page } from "@playwright/test";

import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";

/** Where the screenshots of the main window go (checkpoint 2): ignored by git. */
const SHOTS = "e2e-screenshots/main-window";

/** The app on the sample document of the fake backend, with still motion and the fonts loaded. */
async function openApp(page: Page): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.getByRole("tree", { name: texts.tree.label })).toBeVisible();
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

/** An event of the backend, through the event plugin of the fake IPC. */
async function emitEvent(page: Page, event: string, payload: unknown): Promise<void> {
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

test.describe("the main window", () => {
  test("shows the toolbar, the tree and the vault of the document", async ({ page }) => {
    await openApp(page);

    await expect(page.getByText("portolano.json")).toBeVisible();
    await expect(page.getByRole("button", { name: texts.app.toolbar.newConnection })).toBeVisible();
    await expect(page.getByRole("treeitem", { name: "Clienti" })).toBeVisible();
    await expect(page.getByText(texts.components.vault.fileLocked)).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/main-window.png`, animations: "disabled" });
  });

  test("uses the fonts shipped with the app", async ({ page }) => {
    await openApp(page);

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

    const importButton = page.getByRole("button", { name: texts.app.toolbar.import });
    await expect(importButton).toHaveAttribute("aria-disabled", "true");
    await importButton.hover();
    await expect(page.getByRole("tooltip")).toHaveText(texts.app.reasons.unavailable);
  });

  test("connects to an SSH connection in a tab with its terminal", async ({ page }) => {
    await openApp(page);

    await page.getByRole("treeitem", { name: "bastion" }).dblclick();

    const tab = page.getByRole("tab", { name: /bastion.*SSH/ });
    await expect(tab).toHaveAttribute("aria-selected", "true");
    const panel = page.getByRole("tabpanel");
    await expect(panel.getByRole("status").first()).toHaveText(texts.sessions.status.connected);
    await expect(panel).toContainText("Last login");
    await page.screenshot({ path: `${SHOTS}/ssh-session.png`, animations: "disabled" });
  });

  test("brings the open session in front instead of opening another", async ({ page }) => {
    await openApp(page);
    await page.getByRole("treeitem", { name: "bastion" }).dblclick();
    const session = page.getByRole("tab", { name: /bastion.*SSH/ });
    await expect(session).toBeVisible();
    await page.getByRole("tab", { name: /dettaglio/ }).click();

    await page.getByRole("treeitem", { name: "bastion" }).dblclick();

    await expect(session).toHaveCount(1);
    await expect(session).toHaveAttribute("aria-selected", "true");
  });

  test("closes the session tab with ⌘W and goes back to the detail", async ({ page }) => {
    await openApp(page);
    await page.getByRole("treeitem", { name: "bastion" }).dblclick();
    await expect(page.getByRole("tab", { name: /bastion.*SSH/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await page.getByRole("tab", { name: /bastion.*SSH/ }).focus();
    await page.keyboard.press("Meta+KeyW");

    await expect(page.getByRole("tab", { name: /SSH/ })).toHaveCount(0);
  });

  test("offers to reload a document changed by another program", async ({ page }) => {
    await openApp(page);

    await emitEvent(page, "document-modified-externally", { fileName: "portolano.json" });

    const banner = page.getByRole("alert").filter({ hasText: "portolano.json" });
    await expect(banner).toContainText(
      interpolate(texts.app.banner.modified, { file: "portolano.json" }),
    );
    await page.screenshot({ path: `${SHOTS}/modified-banner.png`, animations: "disabled" });
    await banner.getByRole("button", { name: texts.app.banner.reload }).click();
    await expect(banner).toHaveCount(0);
  });

  test("offers to recreate or open another document when the file is gone", async ({ page }) => {
    await openApp(page);

    await emitEvent(page, "document-missing", { fileName: "portolano.json" });

    const banner = page.getByRole("alert").filter({ hasText: "portolano.json" });
    await expect(banner.getByRole("button", { name: texts.app.banner.recreate })).toBeVisible();
    await expect(banner.getByRole("button", { name: texts.app.banner.openOther })).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/missing-banner.png`, animations: "disabled" });
  });

  test("performs the items of the native menu", async ({ page }) => {
    await openApp(page);

    await emitEvent(page, "menu-action", { command: { kind: "densityCompact" } });

    await expect(page.locator("html")).toHaveAttribute("data-density", "compact");
  });
});
