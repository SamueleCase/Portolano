import type { Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { TEST_MASTER_PASSWORD } from "../src/test/fakeBackendModel";
import { EDITOR_SHOTS, ipcAnswers, recordIpcAnswers } from "./editorPage";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row } from "./mainWindowPage";

const t = texts.editor;
const d = texts.dialogs;
/** A sentinel secret: it must never reach the page, the fields or an answer of the backend. */
const SENTINEL = "sentinel-e2e-window-secret-7e2d";

/** A screenshot of the whole window, for the visual checkpoint 2. */
async function shoot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: `${EDITOR_SHOTS}/window-${name}.png`, animations: "disabled" });
}

/** The editor of node `name`, opened from its detail. */
async function editFromDetail(page: Page, name: string) {
  await row(await documentTree(page), name).click();
  await page.getByRole("button", { name: texts.details.actions.edit, exact: true }).click();
  return page.getByRole("dialog", { name: new RegExp(name) });
}

/** Stages `value` as the password of the credential in the editor `panel`. */
async function stagePassword(page: Page, value: string) {
  const panel = page.getByRole("dialog", { name: /deploy/ });
  await panel.getByRole("button", { name: t.next }).click();
  await panel
    .getByRole("group", { name: t.credential.password })
    .getByRole("button", { name: texts.components.secretRow.set })
    .click();
  const dialog = page.getByRole("dialog", { name: d.secret.titles.password });
  await dialog.getByLabel(d.secret.fields.password).fill(value);
  await dialog.getByLabel(d.secret.confirm).fill(value);
  await dialog.getByRole("button", { name: d.secret.stage }).click();
  await expect(panel.getByText(texts.components.secretBadge.staged)).toBeVisible();
  return panel;
}

/** The sentinel is nowhere: page, values of the fields, answers of the fake backend. */
async function expectNoSentinel(page: Page) {
  expect(await page.content()).not.toContain(SENTINEL);
  const values = await page
    .locator("input, textarea")
    .evaluateAll((fields) => fields.map((field) => ("value" in field ? String(field.value) : "")));
  expect(values).not.toContain(SENTINEL);
  expect(await ipcAnswers(page)).not.toContain(SENTINEL);
}

test.describe("the editor in the main window", () => {
  test("creates an SSH connection from + Connessione and shows it in the tree", async ({
    page,
  }) => {
    await openApp(page, { vaultLocked: false });
    const tree = await documentTree(page);
    await row(tree, "ACME").click();

    await page.getByRole("button", { name: texts.app.toolbar.newConnection }).click();
    await page.getByRole("menuitem", { name: texts.app.toolbar.protocols.ssh }).click();
    const panel = page.getByRole("dialog", { name: t.subtitles.add.ssh });
    await panel.getByLabel(t.general.name).fill("web09");
    await expect(panel.getByText("Clienti / ACME")).toBeVisible();
    await shoot(page, "editor-new-ssh");
    await panel.getByRole("button", { name: t.next }).click();
    await panel.getByLabel(t.connection.host, { exact: true }).fill("web09.example.com");
    await panel.getByRole("button", { name: t.save }).click();

    await expect(panel).toBeHidden();
    await expect(row(tree, "web09")).toBeVisible();
    await expect(row(tree, "web09")).toHaveAttribute("aria-selected", "true");
  });

  test("creates an RDP connection at the root", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    const tree = await documentTree(page);

    await page.getByRole("button", { name: texts.app.toolbar.newConnection }).click();
    await page.getByRole("menuitem", { name: texts.app.toolbar.protocols.rdp }).click();
    const panel = page.getByRole("dialog", { name: t.subtitles.add.rdp });
    await panel.getByLabel(t.general.name).fill("dc09");
    await panel.getByRole("button", { name: t.next }).click();
    await panel.getByLabel(t.connection.host, { exact: true }).fill("192.0.2.90");
    await shoot(page, "editor-new-rdp");
    await panel.getByRole("button", { name: t.save }).click();

    await expect(row(tree, "dc09")).toBeVisible();
  });

  test("saves a staged password only on Salva, and it reaches no answer", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    await documentTree(page);
    await recordIpcAnswers(page);
    await editFromDetail(page, "deploy");

    const panel = await stagePassword(page, SENTINEL);
    await shoot(page, "editor-staged-secret");
    await panel.getByRole("button", { name: t.save }).click();

    await expect(panel).toBeHidden();
    const secrets = page.getByRole("region", { name: texts.details.secrets.heading });
    await expect(secrets.getByRole("group", { name: "Password" })).toContainText(
      texts.components.secretBadge.present.file,
    );
    await expectNoSentinel(page);
  });

  test("asks to unlock the vault over the editor on Salva, then saves", async ({ page }) => {
    await openApp(page);
    await documentTree(page);
    await recordIpcAnswers(page);
    await editFromDetail(page, "deploy");
    const panel = await stagePassword(page, SENTINEL);

    await panel.getByRole("button", { name: t.save }).click();
    const unlock = page.getByRole("dialog", { name: d.unlock.title });
    await expect(unlock).toBeVisible();
    await expect(panel).toBeVisible();
    await shoot(page, "editor-unlock");
    await unlock.getByLabel(d.unlock.field).fill(TEST_MASTER_PASSWORD);
    await unlock.getByRole("button", { name: d.unlock.unlock }).click();

    await expect(panel).toBeHidden();
    await expect(page.getByText(texts.components.vault.fileUnlocked)).toBeVisible();
    await expectNoSentinel(page);
  });
});

test.describe("the dialogs in the main window", () => {
  test("set a password from the detail with Sostituisci…", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    await row(await documentTree(page), "Admin ACME").click();

    const secrets = page.getByRole("region", { name: texts.details.secrets.heading });
    await secrets.getByRole("button", { name: texts.details.secrets.replace }).click();
    const dialog = page.getByRole("dialog", { name: d.secret.replaceTitles.password });
    await dialog.getByLabel(d.secret.fields.password).fill("una-password-di-prova");
    await dialog.getByLabel(d.secret.confirm).fill("una-password-di-prova");
    await shoot(page, "password-from-detail");
    await dialog.getByRole("button", { name: d.secret.save.file }).click();

    await expect(dialog).toBeHidden();
  });

  test("ask the credentials at launch and open the session", async ({ page }) => {
    await openApp(page, { vaultLocked: false });
    const tree = await documentTree(page);
    await row(tree, "ACME").click();
    await page.keyboard.press("ArrowRight");

    await row(tree, "kiosk").dblclick();
    const login = page.getByRole("dialog", { name: d.login.title });
    await login.getByLabel(d.login.user).fill("operatore");
    await login.getByLabel(d.login.domain).fill("ACME");
    await login.getByLabel(d.login.password).fill("una-password-di-prova");
    await shoot(page, "login-at-launch");
    await login.getByRole("button", { name: d.login.connect }).click();

    await expect(login).toBeHidden();
    await expect(page.getByRole("tab", { name: /kiosk.*(SSH|RDP)/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("apply the density of the preferences to the window", async ({ page }) => {
    await openApp(page);
    await documentTree(page);

    await page.getByRole("button", { name: texts.app.toolbar.preferences }).click();
    const dialog = page.getByRole("dialog", { name: texts.preferences.title });
    await dialog
      .getByRole("radiogroup", { name: texts.preferences.appearance.density })
      .getByRole("radio", { name: texts.preferences.appearance.densities.compact })
      .click();

    await expect(page.locator("html")).toHaveAttribute("data-density", "compact");
    await shoot(page, "preferences");
    await dialog.getByRole("button", { name: texts.preferences.close }).click();
    await expect(dialog).toBeHidden();
  });
});
