import type { Locator, Page } from "@playwright/test";

import { importGallery } from "../src/i18n/import";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { openGallery } from "./galleryPage";

/**
 * The steps of the import wizard in the gallery, on fixed data (DEC-037.1):
 * the flows with the backend are tested by Vitest; the complete flows in the
 * main window come with phase 2.
 */

/** Where the screenshots of the wizard go: outside `src/`, ignored by git. */
const SHOTS = "e2e-screenshots/import";

async function specimen(page: Page, caption: string): Promise<Locator> {
  await openGallery(page);
  const figure = page.getByRole("figure", { name: caption });
  await figure.scrollIntoViewIfNeeded();
  return figure;
}

async function shoot(figure: Locator, name: string): Promise<void> {
  await figure.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}

/** The line where the gallery tells where "Vai al nodo" would lead. */
function wentTo(page: Page): Locator {
  return page.locator("section#importValidation").getByRole("status").last();
}

test.describe("the import wizard", () => {
  test("shows a readable file with the objects read", async ({ page }) => {
    const figure = await specimen(page, importGallery.fileRead);

    await expect(figure.getByText(texts.importWizard.file.readable)).toBeVisible();
    await expect(figure.getByText("17 elementi letti")).toBeVisible();
    await shoot(figure, "1-file");
  });

  test("shows the read error of an unreadable file", async ({ page }) => {
    const figure = await specimen(page, importGallery.fileUnreadable);

    await expect(figure.getByRole("alert")).toContainText("Permission denied");
    await shoot(figure, "1-file-unreadable");
  });

  test("changes the sentence of the conflicts with the choice", async ({ page }) => {
    const figure = await specimen(page, importGallery.options);
    const { conflictChoices, conflictHints } = texts.importWizard.options;

    await figure.getByRole("radio", { name: conflictChoices.newIds }).click();

    await expect(figure.getByText(conflictHints.newIds)).toBeVisible();
    await figure.getByRole("radio", { name: conflictChoices.update }).click();
    await shoot(figure, "2-options");
  });

  test("asks a password with N / M and the node", async ({ page }) => {
    const figure = await specimen(page, importGallery.password);

    await expect(figure.getByText("2 / 4")).toBeVisible();
    await expect(figure.getByText("Sede/Produzione/db20")).toBeVisible();
    await shoot(figure, "3-password");
  });

  test("leads from a row of the report to its node", async ({ page }) => {
    const figure = await specimen(page, importGallery.report);

    await figure.getByRole("button", { name: /Sede\/Produzione\/web20/ }).click();

    await expect(wentTo(page)).toHaveText("Vai al nodo: Sede/Produzione/web20");
  });

  test("filters the unsupported items of the report", async ({ page }) => {
    const figure = await specimen(page, importGallery.report);

    await figure
      .getByRole("button", { name: texts.importWizard.report.filters.unsupported })
      .click();

    await expect(figure.getByRole("listitem").filter({ hasText: "Sede/" })).toHaveCount(1);
  });

  test("counts the outcomes, a skipped password as partial", async ({ page }) => {
    const figure = await specimen(page, importGallery.report);

    await expect(figure.getByRole("button", { name: /Sede\/Produzione\/web20/ })).toContainText(
      texts.importWizard.report.outcome.partial,
    );
    await expect(figure.getByText("3 password saltate", { exact: false })).toBeVisible();
    await shoot(figure, "4-report");
  });
});
