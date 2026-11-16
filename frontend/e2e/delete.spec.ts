import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row, shoot } from "./mainWindowPage";

const t = texts.details.remove;

test.describe("the deletion", () => {
  test("of a node nobody uses can be undone from its toast", async ({ page }) => {
    await openApp(page);
    await row(await documentTree(page), "vnc-legacy").click();

    await page.getByRole("button", { name: texts.details.actions.delete }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toContainText(t.unused);
    await dialog.getByRole("button", { name: t.delete, exact: true }).click();

    await expect(row(page, "vnc-legacy")).toHaveCount(0);
    const toast = page.getByRole("listitem").filter({
      hasText: interpolate(texts.app.toasts.deleted, { name: "vnc-legacy" }),
    });
    await expect(toast).toBeVisible();
    await shoot(page, "deleted-toast");
    await toast.getByRole("button", { name: texts.app.toasts.undo }).click();
    await expect(row(page, "vnc-legacy")).toBeVisible();
  });

  test("is final after 10 seconds: the toast goes and the node stays deleted", async ({ page }) => {
    await page.clock.install();
    await openApp(page);
    await row(await documentTree(page), "vnc-legacy").click();
    await page.keyboard.press("Meta+Backspace");
    await page.getByRole("dialog").getByRole("button", { name: t.delete, exact: true }).click();
    const undo = page.getByRole("button", { name: texts.app.toasts.undo });
    await expect(undo).toBeVisible();

    await page.clock.fastForward(11_000);

    await expect(undo).toHaveCount(0);
    await page.getByRole("searchbox", { name: texts.tree.search }).fill("vnc");
    await expect(page.getByText(texts.tree.noResults)).toBeVisible();
  });

  test("of a used credential lists who refers to it and who inherits it", async ({ page }) => {
    await openApp(page);
    await row(await documentTree(page), "Admin ACME").click({ button: "right" });

    await page.getByRole("menuitem", { name: texts.tree.menu.delete }).click();

    const dialog = page.getByRole("dialog", {
      name: interpolate(t.titleCredential, { name: "Admin ACME" }),
    });
    await expect(dialog).toContainText(/È usata da 3 nodi/);
    await expect(dialog).toContainText(t.folderDefault);
    await expect(dialog).toContainText(/La ricevono per ereditarietà 2 nodi/);
    await expect(dialog).toContainText(t.credentialText);
    await expect(dialog.getByRole("button", { name: t.deleteAnyway })).toBeVisible();
    await shoot(page, "delete-confirm");
  });

  test("goes to a node that uses the credential from the confirmation", async ({ page }) => {
    await openApp(page);
    await row(await documentTree(page), "Admin ACME").click();
    await page.getByRole("button", { name: texts.details.actions.delete }).click();

    await page
      .getByRole("dialog")
      .getByRole("button", { name: /Clienti \/ ts01/ })
      .click();

    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(row(page, "ts01")).toHaveAttribute("aria-selected", "true");
  });

  test("is cancelled by Enter, the default action", async ({ page }) => {
    await openApp(page);
    await row(await documentTree(page), "Admin ACME").click();
    await page.getByRole("button", { name: texts.details.actions.delete }).click();
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.keyboard.press("Enter");

    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(row(page, "Admin ACME")).toBeVisible();
  });
});
