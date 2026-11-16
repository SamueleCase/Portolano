import type { Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row, shoot } from "./mainWindowPage";

const t = texts.details;

/** Shows the detail of a node reached through the closed folder ACME. */
async function showInAcme(page: Page, name: string): Promise<void> {
  const tree = await documentTree(page);
  await row(tree, "ACME").click();
  await row(tree, name).click();
}

test.describe("the detail", () => {
  test("shows the effective credential with the chain of its folders", async ({ page }) => {
    await openApp(page);

    await showInAcme(page, "web01");

    const credential = page.getByRole("group", { name: t.credential.heading });
    await expect(credential).toContainText("Ereditata dalla cartella ACME");
    await expect(credential.getByRole("list", { name: t.credential.chain })).toContainText(
      "Admin ACME",
    );
    await expect(credential).toContainText(texts.components.secretBadge.locked);
    await expect(page.getByRole("region", { name: texts.components.route.label })).toContainText(
      "bastion.example.com",
    );
    await shoot(page, "detail");
  });

  test("shows the command of the real plan with «Mostra comando»", async ({ page }) => {
    await openApp(page);
    await showInAcme(page, "web01");

    await page.getByRole("button", { name: t.actions.showCommand }).click();

    const command = page.getByRole("region", { name: t.command.heading });
    await expect(command).toContainText("-p 2222");
    await expect(command).toContainText(t.command.caption.file);
    await shoot(page, "detail-command");
    await page.getByRole("button", { name: t.actions.hideCommand }).click();
    await expect(command).toHaveCount(0);
  });

  test("says a credential found by name was looked up by name", async ({ page }) => {
    await openApp(page);

    await showInAcme(page, "db01");

    await expect(page.getByRole("group", { name: t.credential.heading })).toContainText(
      t.credential.origins.byName,
    );
  });

  test("says why a credential looked up by a missing name does not resolve", async ({ page }) => {
    await openApp(page, { unresolvedCredential: true });

    await showInAcme(page, "web03");

    const credential = page.getByRole("group", { name: t.credential.heading });
    await expect(credential).toContainText(t.credential.origins.unresolved);
    await expect(credential).toContainText("Credenziale assente");
    await shoot(page, "detail-unresolved");
  });

  test("shows a folder with its content and its default credential", async ({ page }) => {
    await openApp(page);

    await row(await documentTree(page), "ACME").click();

    await expect(page.getByText("4 elementi").last()).toBeVisible();
    await expect(page.getByRole("group", { name: t.credential.heading })).toContainText(
      t.credential.origins.folderDefault,
    );
    await shoot(page, "detail-folder");
  });

  test("lists who uses a credential and goes to them", async ({ page }) => {
    await openApp(page);

    await row(await documentTree(page), "Admin ACME").click();

    const usage = page.getByRole("region", { name: t.usage.heading });
    await expect(usage).toContainText("Clienti / ACME");
    await usage
      .getByRole("button", { name: /Clienti \/ ACME\s/ })
      .first()
      .click();
    await expect(row(page, "ACME")).toHaveAttribute("aria-selected", "true");
    await expect(row(page, "ACME")).toHaveAttribute("tabindex", "0");
  });

  test("shows an unsupported node read-only, with its original as text", async ({ page }) => {
    await openApp(page);

    await row(await documentTree(page), "vnc-legacy").click();

    await expect(page.getByText("RoyalVNCConnection").first()).toBeVisible();
    await expect(page.getByRole("region", { name: t.unsupported.raw })).toContainText(
      "<RoyalVNCConnection",
    );
    await expect(page.getByRole("button", { name: t.actions.edit })).toHaveCount(0);
    await expect(page.getByRole("button", { name: t.actions.duplicate })).toHaveCount(0);
    await shoot(page, "detail-unsupported");
  });

  test("duplicates a node and selects the copy", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    await showInAcme(page, "web01");

    await page.getByRole("button", { name: t.actions.duplicate, exact: true }).click();

    await expect(row(page, "web01 (copia)")).toHaveAttribute("aria-selected", "true");
    await expect(
      page
        .getByRole("listitem")
        .filter({ hasText: "Copia creata: Clienti / ACME / web01 (copia)" }),
    ).toBeVisible();
  });

  test("tells the renamed credentials of a copy", async ({ page }) => {
    await openApp(page, { vaultLocked: false });

    await row(await documentTree(page), "Admin ACME").click();
    await page.getByRole("button", { name: t.actions.duplicate, exact: true }).click();

    await expect(
      page.getByRole("listitem").filter({ hasText: /La credenziale copiata è stata rinominata/ }),
    ).toBeVisible();
    await shoot(page, "duplicate-renamed");
  });
});
