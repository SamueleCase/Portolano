import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { documentTree, openApp } from "./mainWindowPage";

/**
 * The command of a custom terminal in the preferences of the main window: a
 * new one is saved only once the user confirms it in the native dialog of
 * the backend (DEC-046.1). The fake backend answers that dialog as the seed
 * says.
 */

const t = texts.preferences;

test.describe("the command of a custom terminal", () => {
  test("goes back to the saved one, without an error, when the user does not confirm it", async ({
    page,
  }) => {
    await openApp(page, { dialogs: { terminalCommand: false } });
    await documentTree(page);
    await page.getByRole("button", { name: texts.app.toolbar.preferences }).click();
    const dialog = page.getByRole("dialog", { name: t.title });
    await dialog
      .getByRole("radiogroup", { name: t.terminal.label })
      .getByRole("radio", { name: t.terminal.apps.custom })
      .click();
    const field = dialog.getByLabel(t.terminal.command);

    await field.fill("/opt/example/bin/altro {cmd}");
    await field.press("Enter");

    await expect(field).toHaveValue("");
    await expect(dialog.getByRole("alert")).toHaveCount(0);
  });

  test("is saved when the user confirms it", async ({ page }) => {
    await openApp(page);
    await documentTree(page);
    await page.getByRole("button", { name: texts.app.toolbar.preferences }).click();
    const dialog = page.getByRole("dialog", { name: t.title });
    await dialog
      .getByRole("radiogroup", { name: t.terminal.label })
      .getByRole("radio", { name: t.terminal.apps.custom })
      .click();
    const field = dialog.getByLabel(t.terminal.command);

    await field.fill("wezterm start -- {cmd}");
    await field.press("Enter");
    await dialog.getByRole("button", { name: t.close }).click();
    await page.getByRole("button", { name: texts.app.toolbar.preferences }).click();

    await expect(dialog.getByLabel(t.terminal.command)).toHaveValue("wezterm start -- {cmd}");
  });
});
