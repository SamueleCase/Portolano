import type { Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import type { FakeSeed } from "../src/test/fakeBackend";
import { expect, test } from "./fixtures";

const SHOTS = "e2e-screenshots/main-window";
const t = texts.details;

async function openApp(page: Page, seed: FakeSeed = {}): Promise<void> {
  await page.addInitScript((value) => {
    window.__portolanoFakeSeed = value;
  }, seed);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.getByRole("tree", { name: texts.tree.label })).toBeVisible();
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

/** Shows the detail of a node reached through the closed folder ACME. */
async function showInAcme(page: Page, name: string): Promise<void> {
  await page.getByRole("treeitem", { name: "ACME", exact: true }).click();
  await page.getByRole("treeitem", { name }).click();
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
    await page.screenshot({ path: `${SHOTS}/detail.png`, animations: "disabled" });
  });

  test("shows the command of the real plan with «Mostra comando»", async ({ page }) => {
    await openApp(page);
    await showInAcme(page, "web01");

    await page.getByRole("button", { name: t.actions.showCommand }).click();

    const command = page.getByRole("region", { name: t.command.heading });
    await expect(command).toContainText("-p 2222");
    await expect(command).toContainText(t.command.caption.file);
    await page.screenshot({ path: `${SHOTS}/detail-command.png`, animations: "disabled" });
    await page.getByRole("button", { name: t.actions.hideCommand }).click();
    await expect(command).toHaveCount(0);
  });

  test("says when a credential by name is ambiguous or missing", async ({ page }) => {
    await openApp(page);

    await showInAcme(page, "db01");

    await expect(page.getByRole("group", { name: t.credential.heading })).toContainText(
      t.credential.origins.byName,
    );
  });

  test("lists who uses a credential and goes to them", async ({ page }) => {
    await openApp(page);

    await page.getByRole("treeitem", { name: "Admin ACME" }).click();

    const usage = page.getByRole("region", { name: t.usage.heading });
    await expect(usage).toContainText("Clienti / ACME");
    await usage
      .getByRole("button", { name: /Clienti \/ ACME\s/ })
      .first()
      .click();
    await expect(page.getByRole("treeitem", { name: "ACME", exact: true })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("shows an unsupported node read-only, with its original as text", async ({ page }) => {
    await openApp(page);

    await page.getByRole("treeitem", { name: "vnc-legacy" }).click();

    await expect(page.getByText("RoyalVNCConnection").first()).toBeVisible();
    await expect(page.getByLabel(t.unsupported.raw)).toContainText("<RoyalVNCConnection");
    await expect(page.getByRole("button", { name: t.actions.edit })).toHaveCount(0);
    await page.screenshot({ path: `${SHOTS}/detail-unsupported.png`, animations: "disabled" });
  });

  test("duplicates a node and selects the copy", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    await showInAcme(page, "web01");

    await page.getByRole("button", { name: t.actions.duplicate, exact: true }).click();

    await expect(page.getByRole("treeitem", { name: "web01 (copia)" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(
      page
        .getByRole("listitem")
        .filter({ hasText: "Copia creata: Clienti / ACME / web01 (copia)" }),
    ).toBeVisible();
  });

  test("tells the renamed credentials of a copy", async ({ page }) => {
    await openApp(page, { vaultLocked: false });

    await page.getByRole("treeitem", { name: "Admin ACME" }).click();
    await page.getByRole("button", { name: t.actions.duplicate, exact: true }).click();

    await expect(
      page.getByRole("listitem").filter({ hasText: /La credenziale copiata è stata rinominata/ }),
    ).toBeVisible();
  });
});
