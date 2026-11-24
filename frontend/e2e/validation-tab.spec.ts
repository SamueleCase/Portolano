import type { Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import type { FakeSeed } from "../src/test/fakeBackend";
import { expect, test } from "./fixtures";
import { documentTree, openApp, row, sidebarTool } from "./mainWindowPage";

/**
 * The validation tab in the main window, on the fake backend started by the
 * seed: "Valida", the rows that lead to their node, "Ricontrolla" and the
 * note of the locked vault.
 */

/** Where the screenshots of the validation go (checkpoint 2): outside `src/`, ignored by git. */
const SHOTS = "e2e-screenshots/validation";

async function openValidation(page: Page, seed: FakeSeed = {}) {
  await openApp(page, seed);
  await documentTree(page);
  await sidebarTool(page, "validate").click();
  const tab = page.getByRole("tab", { name: /Validazione/ });
  await expect(tab).toHaveAttribute("aria-selected", "true");
  const panel = page.getByRole("tabpanel");
  await expect(panel.getByRole("heading", { level: 2 })).toBeVisible();
  return { tab, panel };
}

function rows(page: Page) {
  return page
    .getByRole("tabpanel")
    .getByRole("list", { name: texts.validation.list, exact: true })
    .getByRole("listitem");
}

async function shoot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}

/** How many times the page asked the fake backend to validate. */
async function validations(page: Page): Promise<number> {
  return page.evaluate(() => Number(Reflect.get(window, "__validations") ?? 0));
}

test.describe("the validation tab in the main window", () => {
  test("opens with Valida and counts the problems in the tab", async ({ page }) => {
    const { tab, panel } = await openValidation(page, { vaultLocked: false });

    await expect(panel.getByRole("heading", { name: "2 problemi nel documento" })).toBeVisible();
    await expect(tab).toContainText("2 problemi");
    await shoot(page, "window-problems");
  });

  test("leads from a row to its node in the tree and its detail", async ({ page }) => {
    await openValidation(page, { vaultLocked: false });

    await rows(page)
      .filter({ hasText: "vnc-legacy" })
      .getByRole("button", { name: texts.components.validation.goToNode })
      .click();

    await expect(row(page, "vnc-legacy")).toHaveAttribute("aria-selected", "true");
    await expect(page.getByRole("tab", { name: /vnc-legacy/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("checks the document again with Ricontrolla", async ({ page }) => {
    const { panel } = await openValidation(page, { vaultLocked: false });
    await page.evaluate(() => {
      // Counts the calls of `validate` on the fake IPC, a global of Tauri without types.
      const internals = Reflect.get(window, "__TAURI_INTERNALS__") as {
        invoke: (command: string, args: unknown, options?: unknown) => Promise<unknown>;
      };
      const original = internals.invoke.bind(internals);
      internals.invoke = (command, args, options) => {
        if (command === "validate") {
          Reflect.set(
            window,
            "__validations",
            Number(Reflect.get(window, "__validations") ?? 0) + 1,
          );
        }
        return original(command, args, options);
      };
    });

    await panel.getByRole("button", { name: texts.validation.recheck }).click();

    await expect.poll(() => validations(page)).toBe(1);
    await expect(panel.getByRole("heading", { name: "2 problemi nel documento" })).toBeVisible();
  });

  test("shows only the note of the locked vault for the secrets", async ({ page }) => {
    await openValidation(page, { vaultLocked: true });

    const note = rows(page).filter({ hasText: "il vault è bloccato" });
    await expect(note).toHaveCount(1);
    await expect(note.getByRole("button")).toHaveCount(0);
    await shoot(page, "window-vault-locked");
  });
});
