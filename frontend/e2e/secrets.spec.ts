import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import { IMPORT_FILE } from "../src/test/fakeBackendImport";
import { TEST_MASTER_PASSWORD } from "../src/test/fakeBackendModel";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row } from "./mainWindowPage";
import { SecretWatch } from "./secretsPage";

/**
 * The sentinel secret (SPEC-UI §1.2, §7): every flow of the main window that
 * receives a secret typed by the user sends it to the backend once, and
 * then it is nowhere in the page: not in an answer, an error or an event of
 * the fake backend, not in the DOM nor in a field, not in the storage, the
 * address or the console, not in the cache of TanStack Query.
 */

const t = texts.editor;
const d = texts.dialogs;
const secretRow = texts.components.secretRow;

/** A distinct sentinel for each flow, so that a failure names its flow. */
const SENTINEL = {
  editorPassword: "sentinel-editor-password-41c7",
  editorKey: "sentinel-editor-key-9b02",
  editorPassphrase: "sentinel-editor-passphrase-5d18",
  detailPassword: "sentinel-detail-password-2e6a",
  login: "sentinel-login-password-c3f0",
  wrongMaster: "sentinel-wrong-master-password-77ad",
  newMaster: "sentinel-new-master-password-0f9e",
  importPassword: "sentinel-import-password-a61b",
} as const;

/** A private key in the OpenSSH form, with the sentinel on one of its lines. */
const PRIVATE_KEY = [
  "-----BEGIN OPENSSH PRIVATE KEY-----",
  SENTINEL.editorKey,
  "-----END OPENSSH PRIVATE KEY-----",
].join("\n");

/** The secrets of a credential in the editor, with the name of their setting. */
const SETTINGS = {
  password: t.credential.password,
  privateKey: t.credential.key,
  passphrase: t.credential.passphrase,
} as const;

/**
 * Stages `value` as the secret `slot` of the credential in the editor
 * `panel`: "Imposta…" opens the dialog, titled "Imposta" or "Sostituisci"
 * whether the document already refers to a secret there.
 */
async function stage(
  page: Page,
  panel: Locator,
  slot: keyof typeof SETTINGS,
  value: string,
): Promise<void> {
  await panel
    .getByRole("group", { name: SETTINGS[slot] })
    .getByRole("button", { name: secretRow.set })
    .click();
  const dialog = page
    .getByRole("dialog", { name: d.secret.titles[slot] })
    .or(page.getByRole("dialog", { name: d.secret.replaceTitles[slot] }));
  await dialog.getByLabel(d.secret.fields[slot]).fill(value);
  if (slot !== "privateKey") await dialog.getByLabel(d.secret.confirm).fill(value);
  await dialog.getByRole("button", { name: d.secret.stage }).click();
  await expect(dialog).toBeHidden();
}

/** The footer of the file vault, which opens its unlock or its creation. */
function vaultFooter(page: Page): Locator {
  return page.getByRole("button", { name: texts.components.vault.fileLocked });
}

test.describe("a typed secret", () => {
  test("staged in the editor goes to the backend on Salva and stays nowhere", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page, { vaultLocked: false });
    await row(await documentTree(page), "deploy").click();
    await page.getByRole("button", { name: texts.details.actions.edit, exact: true }).click();
    const panel = page.getByRole("dialog", { name: /deploy/ });
    // A value that is not secret: it must come back, so the watch sees the answers.
    const description = "descrizione-di-controllo-8c4e";
    await panel.getByLabel(t.general.description).fill(description);
    await panel.getByRole("button", { name: t.next }).click();

    await stage(page, panel, "password", SENTINEL.editorPassword);
    await panel.getByRole("radio", { name: t.credential.keyModes.vault.file }).click();
    await stage(page, panel, "privateKey", PRIVATE_KEY);
    await stage(page, panel, "passphrase", SENTINEL.editorPassphrase);
    await panel.getByRole("button", { name: t.save }).click();
    await expect(panel).toBeHidden();
    await expect(page.getByText(description)).toBeVisible();
    // The three secrets reached the vault of the backend.
    const secrets = page.getByRole("region", { name: texts.details.secrets.heading });
    for (const slot of Object.values(texts.details.secrets.slots)) {
      await expect(secrets.getByRole("group", { name: slot })).toContainText(
        texts.components.secretBadge.present.file,
      );
    }

    const surfaces = await watch.surfaces();
    expect(surfaces.traffic).toContain(description);
    expect(surfaces.queryCache).toContain(description);
    for (const secret of [SENTINEL.editorPassword, SENTINEL.editorKey, SENTINEL.editorPassphrase]) {
      await watch.expectNowhere(secret);
    }
  });

  test("in the password dialog of the detail stays nowhere once saved", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page, { vaultLocked: false });
    await row(await documentTree(page), "Admin ACME").click();

    await page
      .getByRole("region", { name: texts.details.secrets.heading })
      .getByRole("button", { name: texts.details.secrets.replace })
      .click();
    const dialog = page.getByRole("dialog", { name: d.secret.replaceTitles.password });
    await dialog.getByLabel(d.secret.fields.password).fill(SENTINEL.detailPassword);
    await dialog.getByLabel(d.secret.confirm).fill(SENTINEL.detailPassword);
    await dialog.getByRole("button", { name: d.secret.save.file }).click();
    await expect(dialog).toBeHidden();

    // The backend stored it: its answer says the password is present.
    expect((await watch.surfaces()).traffic).toMatch(
      /^answer set_secret: .*"slot":"password","state":"present"/m,
    );
    await watch.expectNowhere(SENTINEL.detailPassword);
  });

  test("as the credentials at launch stays nowhere once the session opens", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page);
    const tree = await documentTree(page);
    await row(tree, "ACME").click();
    await page.keyboard.press("ArrowRight");

    await row(tree, "kiosk").dblclick();
    const login = page.getByRole("dialog", { name: d.login.title });
    await login.getByLabel(d.login.user).fill("operatore");
    await login.getByLabel(d.login.password).fill(SENTINEL.login);
    await login.getByRole("button", { name: d.login.connect }).click();
    await expect(login).toBeHidden();
    await expect(page.getByRole("tab", { name: /kiosk.*(SSH|RDP)/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(page.getByRole("tabpanel")).toContainText("Last login");

    // The record holds the answer, the states and the output of the session.
    const { traffic } = await watch.surfaces();
    expect(traffic).toMatch(/^answer open_session: /m);
    expect(traffic).toMatch(/^output: Last login/m);
    await watch.expectNowhere(SENTINEL.login);
  });

  test("as the master password, wrong and then right, stays nowhere", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page);
    await documentTree(page);

    await vaultFooter(page).click();
    const dialog = page.getByRole("dialog", { name: d.unlock.title });
    await dialog.getByLabel(d.unlock.field).fill(SENTINEL.wrongMaster);
    await dialog.getByRole("button", { name: d.unlock.unlock }).click();
    await expect(dialog.getByRole("alert")).toContainText("master password errata");
    await dialog.getByLabel(d.unlock.field).fill(TEST_MASTER_PASSWORD);
    await dialog.getByRole("button", { name: d.unlock.unlock }).click();
    await expect(dialog).toBeHidden();
    await expect(page.getByText(texts.components.vault.fileUnlocked)).toBeVisible();

    // The error of the wrong one reached the record: the record sees the errors.
    expect((await watch.surfaces()).traffic).toMatch(
      /^error unlock_vault: .*master password errata/m,
    );
    await watch.expectNowhere(SENTINEL.wrongMaster);
    await watch.expectNowhere(TEST_MASTER_PASSWORD);
  });

  test("as the master password of a new vault stays nowhere", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page, { vaultMissing: true });
    await documentTree(page);

    await vaultFooter(page).click();
    const dialog = page.getByRole("dialog", { name: d.create.title });
    await dialog.getByLabel(d.create.field).fill(SENTINEL.newMaster);
    await dialog.getByLabel(d.create.confirm).fill(SENTINEL.newMaster);
    await dialog.getByRole("button", { name: d.create.create }).click();
    await expect(dialog).toBeHidden();
    await expect(page.getByText(texts.components.vault.fileUnlocked)).toBeVisible();

    await watch.expectNowhere(SENTINEL.newMaster);
  });

  test("as a password of the import stays nowhere once imported", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page, { vaultLocked: false, dialogs: { importFile: IMPORT_FILE } });
    await documentTree(page);
    const { actions } = texts.importWizard;

    await page.getByRole("button", { name: texts.app.toolbar.import }).click();
    const wizard = page.getByRole("dialog", { name: texts.importWizard.title });
    await wizard.getByRole("button", { name: texts.importWizard.file.choose }).click();
    await expect(wizard.getByText(texts.importWizard.file.readable)).toBeVisible();
    await wizard.getByRole("button", { name: actions.next }).click();
    await expect(wizard.getByText(/^In prova:/)).toBeVisible();
    await wizard.getByRole("button", { name: actions.next }).click();
    await wizard
      .getByLabel(texts.importWizard.passwords.fields.password, { exact: true })
      .fill(SENTINEL.importPassword);
    await wizard.getByRole("button", { name: actions.saveAndContinue }).click();
    await wizard.getByRole("button", { name: actions.skipAll }).click();
    await expect(wizard.getByText(/^Importazione completata/)).toBeVisible();

    // web10, whose password was typed, is imported whole: it is not among
    // the items to check, where db10, whose password was skipped, is.
    await expect(wizard.getByRole("button", { name: /db10/ })).toBeVisible();
    await expect(wizard.getByRole("button", { name: /web10/ })).toHaveCount(0);
    await watch.expectNowhere(SENTINEL.importPassword);
  });

  test("would be found by the watch on every surface it reads", async ({ page }) => {
    const watch = new SecretWatch(page);
    await openApp(page);
    await documentTree(page);
    const probe = "sonda-di-controllo-3a5d";

    await page.evaluate((value) => {
      // A nested object: WebKit writes "{dettagli: Object}" as its text, the
      // value is only in the argument.
      console.info({ dettagli: { sessione: { sonda: value } } });
      sessionStorage.setItem("sonda", value);
      document.body.dataset["sonda"] = value;
      history.replaceState(null, "", `?sonda=${value}`);
    }, probe);
    // The search keeps what was typed in the field and in the key of its query.
    await page.getByRole("searchbox", { name: texts.tree.search }).fill(probe);
    await expect(page.getByText(texts.tree.noResults)).toBeVisible();

    const { traffic, ...surfaces } = await watch.surfaces();
    for (const [surface, text] of Object.entries(surfaces)) {
      expect(text, `la sonda manca in: ${surface}`).toContain(probe);
    }
    // The answers carry no typed value here: a host of the tree shows the record is read.
    expect(traffic).toMatch(/^answer tree: .*bastion\.example\.com/m);
  });
});
