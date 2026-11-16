import type { Page } from "@playwright/test";

import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";

const SHOTS = "e2e-screenshots/main-window";
const t = texts.details.remove;

async function openApp(page: Page): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.getByRole("tree", { name: texts.tree.label })).toBeVisible();
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

test.describe("the deletion", () => {
  test("of a node nobody uses can be undone from its toast", async ({ page }) => {
    await openApp(page);
    await page.getByRole("treeitem", { name: "vnc-legacy" }).click();

    await page.getByRole("button", { name: texts.details.actions.delete }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toContainText(t.unused);
    await dialog.getByRole("button", { name: t.delete, exact: true }).click();

    await expect(page.getByRole("treeitem", { name: "vnc-legacy" })).toHaveCount(0);
    const toast = page.getByRole("listitem").filter({
      hasText: interpolate(texts.app.toasts.deleted, { name: "vnc-legacy" }),
    });
    await toast.getByRole("button", { name: texts.app.toasts.undo }).click();
    await expect(page.getByRole("treeitem", { name: "vnc-legacy" })).toBeVisible();
  });

  test("is final after 10 seconds: the toast goes and the node stays deleted", async ({ page }) => {
    await page.clock.install();
    await openApp(page);
    await page.getByRole("treeitem", { name: "vnc-legacy" }).click();
    await page.keyboard.press("Meta+Backspace");
    await page.getByRole("dialog").getByRole("button", { name: t.delete, exact: true }).click();
    const undo = page.getByRole("button", { name: texts.app.toasts.undo });
    await expect(undo).toBeVisible();

    await page.clock.fastForward(11_000);

    await expect(undo).toHaveCount(0);
    await page.getByRole("searchbox", { name: texts.tree.search }).fill("vnc");
    await expect(page.getByText(texts.tree.noResults)).toBeVisible();
  });

  test("of a used credential lists its users and asks «Elimina comunque»", async ({ page }) => {
    await openApp(page);
    await page.getByRole("treeitem", { name: "Admin ACME" }).click({ button: "right" });

    await page.getByRole("menuitem", { name: texts.tree.menu.delete }).click();

    const dialog = page.getByRole("dialog", {
      name: interpolate(t.titleCredential, { name: "Admin ACME" }),
    });
    await expect(dialog).toContainText(/È usata da \d+ nodi/);
    await expect(dialog).toContainText(t.credentialText);
    await expect(dialog.getByRole("button", { name: t.deleteAnyway })).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/delete-confirm.png`, animations: "disabled" });
  });

  test("is cancelled by Enter, the default action", async ({ page }) => {
    await openApp(page);
    await page.getByRole("treeitem", { name: "Admin ACME" }).click();
    await page.getByRole("button", { name: texts.details.actions.delete }).click();
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.keyboard.press("Enter");

    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("treeitem", { name: "Admin ACME" })).toBeVisible();
  });
});
