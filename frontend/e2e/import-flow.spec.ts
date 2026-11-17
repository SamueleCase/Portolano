import type { Locator, Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import type { FakeSeed } from "../src/test/fakeBackend";
import { IMPORT_FILE, UNREADABLE_IMPORT_FILE } from "../src/test/fakeBackendImport";
import { TEST_MASTER_PASSWORD } from "../src/test/fakeBackendModel";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row } from "./mainWindowPage";

/**
 * The import wizard in the main window, on the fake backend started by the
 * seed: "Importa", the native dialog of the file, the options with the
 * destination, the passwords, the report and its rows that lead to the nodes.
 */

/** Where the screenshots of the wizard go (checkpoint 2): outside `src/`, ignored by git. */
const SHOTS = "e2e-screenshots/import";

const SENTINEL = "sentinel-import-e2e-7f31";

const { actions } = texts.importWizard;

const withFile = (file: string, extra: FakeSeed = {}): FakeSeed => ({
  vaultLocked: false,
  ...extra,
  dialogs: { importFile: file, report: "/Users/utente-di-prova/Documenti/report-import.json" },
});

/** Opens the app with `seed` and the wizard over the window. */
async function openWizard(page: Page, seed: FakeSeed): Promise<Locator> {
  await openApp(page, seed);
  await documentTree(page);
  await page.getByRole("button", { name: texts.app.toolbar.import }).click();
  const wizard = page.getByRole("dialog", { name: texts.importWizard.title });
  await expect(wizard).toBeVisible();
  return wizard;
}

async function chooseFile(wizard: Locator): Promise<void> {
  await wizard.getByRole("button", { name: texts.importWizard.file.choose }).click();
}

async function toOptions(wizard: Locator): Promise<void> {
  await chooseFile(wizard);
  await expect(wizard.getByText(texts.importWizard.file.readable)).toBeVisible();
  await wizard.getByRole("button", { name: actions.next }).click();
  await expect(wizard.getByText(/^In prova:/)).toBeVisible();
}

async function shoot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}

/**
 * Records, in the page, every command with its arguments and its answer as
 * text: the fake IPC lives on a global of Tauri, which has no types.
 */
async function recordIpc(page: Page): Promise<void> {
  await page.evaluate(() => {
    const internals = Reflect.get(window, "__TAURI_INTERNALS__") as {
      invoke: (command: string, args: unknown, options?: unknown) => Promise<unknown>;
    };
    const original = internals.invoke.bind(internals);
    const log: { command: string; args: string; answer: string }[] = [];
    Reflect.set(window, "__ipcLog", log);
    internals.invoke = async (command, args, options) => {
      const entry = { command, args: JSON.stringify(args ?? null), answer: "" };
      log.push(entry);
      try {
        const answer = await original(command, args, options);
        entry.answer = JSON.stringify(answer ?? null);
        return answer;
      } catch (failure) {
        entry.answer = JSON.stringify(failure);
        throw failure;
      }
    };
  });
}

async function ipcLog(page: Page): Promise<{ command: string; args: string; answer: string }[]> {
  return page.evaluate(
    () => Reflect.get(window, "__ipcLog") as { command: string; args: string; answer: string }[],
  );
}

test.describe("the import wizard in the main window", () => {
  test("imports a file with the passwords into the chosen folder", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE));
    await shoot(page, "window-1-file");
    await chooseFile(wizard);
    await expect(wizard.getByText("9 elementi letti")).toBeVisible();
    await shoot(page, "window-1-file-read");
    await wizard.getByRole("button", { name: actions.next }).click();
    await expect(wizard.getByText(/^In prova:/)).toBeVisible();

    await wizard
      .getByRole("button", { name: texts.importWizard.options.chooseDestination })
      .click();
    const picker = page.getByRole("dialog", { name: texts.importWizard.options.pickerTitle });
    await picker.getByRole("option", { name: /Rete/ }).click();
    await picker.getByRole("button", { name: "Scegli Rete" }).click();
    await expect(wizard.getByText("Rete / Ufficio")).toBeVisible();
    await shoot(page, "window-2-options");

    await wizard.getByRole("button", { name: actions.next }).click();
    await expect(wizard.getByText("1 / 3")).toBeVisible();
    await shoot(page, "window-3-password");
    await wizard.getByLabel("Password", { exact: true }).fill("password-di-prova");
    await wizard.getByRole("button", { name: actions.saveAndContinue }).click();
    await expect(wizard.getByText("2 / 3")).toBeVisible();
    await wizard.getByRole("button", { name: actions.skip, exact: true }).click();
    await expect(wizard.getByText("3 / 3")).toBeVisible();
    await expect(wizard.getByText("1 inserita · 1 saltata")).toBeVisible();
    await wizard.getByRole("button", { name: actions.skipAll }).click();

    await expect(wizard.getByText(/^Importazione completata/)).toBeVisible();
    await expect(wizard.getByText("2 password saltate", { exact: false })).toBeVisible();
    await shoot(page, "window-4-report");
  });

  test("leads from a row of the report to its node in the tree", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE));
    await toOptions(wizard);
    await wizard.getByRole("button", { name: actions.next }).click();
    await wizard.getByRole("button", { name: actions.skipAll }).click();
    await expect(wizard.getByText(/^Importazione completata/)).toBeVisible();

    await wizard.getByRole("button", { name: /Ufficio \/ Produzione \/ web10/ }).click();

    await expect(row(page, "web10")).toHaveAttribute("aria-selected", "true");
  });

  test("goes to the imported folder with Vai all'albero", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE));
    await toOptions(wizard);
    await wizard.getByRole("switch", { name: texts.importWizard.options.askPasswords }).click();
    await wizard.getByRole("button", { name: actions.next }).click();

    await wizard.getByRole("button", { name: actions.goToTree }).click();

    await expect(wizard).toBeHidden();
    await expect(row(page, "Ufficio")).toHaveAttribute("aria-selected", "true");
    await shoot(page, "window-imported");
  });

  test("shows the read error of an unreadable file", async ({ page }) => {
    const wizard = await openWizard(page, withFile(UNREADABLE_IMPORT_FILE));

    await chooseFile(wizard);

    await expect(wizard.getByRole("alert")).toContainText("Permission denied");
    await expect(wizard.getByRole("button", { name: actions.next })).toHaveAttribute(
      "aria-disabled",
      "true",
    );
    await shoot(page, "window-1-file-unreadable");
  });

  test("opens the locked vault on top of the wizard, then asks the passwords", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE, { vaultLocked: true }));
    await toOptions(wizard);
    await expect(wizard.getByText(/^Il vault è bloccato/)).toBeVisible();
    await shoot(page, "window-2-vault-locked");

    await wizard.getByRole("button", { name: actions.next }).click();
    // The unlock dialog of the window, asked by the wizard, which stays open (DEC-041.1).
    const unlock = page.getByRole("dialog", { name: texts.dialogs.unlock.title });
    await expect(unlock).toBeVisible();
    await shoot(page, "window-2-unlock");
    await unlock.getByLabel(texts.dialogs.unlock.field).fill(TEST_MASTER_PASSWORD);
    await unlock.getByRole("button", { name: texts.dialogs.unlock.unlock }).click();

    await expect(unlock).toBeHidden();
    await expect(wizard.getByText("1 / 3")).toBeVisible();
    await expect(wizard.getByLabel("Password", { exact: true })).toBeFocused();
  });

  test("stays on the options when the vault is left locked", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE, { vaultLocked: true }));
    await toOptions(wizard);
    await wizard.getByRole("button", { name: actions.next }).click();
    const unlock = page.getByRole("dialog", { name: texts.dialogs.unlock.title });

    await unlock.getByRole("button", { name: texts.dialogs.unlock.withoutSecrets }).click();

    await expect(unlock).toBeHidden();
    await expect(wizard.getByText(/^Il vault è bloccato/)).toBeVisible();
    await expect(wizard.getByText("1 / 3")).toBeHidden();
  });

  test("sends a typed password only in import_apply and gets it back nowhere", async ({ page }) => {
    const wizard = await openWizard(page, withFile(IMPORT_FILE));
    await recordIpc(page);
    await toOptions(wizard);
    await wizard.getByRole("button", { name: actions.next }).click();

    await wizard.getByLabel("Password", { exact: true }).fill(SENTINEL);
    await wizard.getByRole("button", { name: actions.saveAndContinue }).click();
    await wizard.getByRole("button", { name: actions.skipAll }).click();
    await expect(wizard.getByText(/^Importazione completata/)).toBeVisible();

    const log = await ipcLog(page);
    const carrying = log.filter((entry) => entry.args.includes(SENTINEL));
    expect(carrying.map((entry) => entry.command)).toEqual(["import_apply"]);
    expect(log.filter((entry) => entry.answer.includes(SENTINEL))).toEqual([]);
    expect(await page.content()).not.toContain(SENTINEL);
  });
});
