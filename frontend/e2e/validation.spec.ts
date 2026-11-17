import type { Locator, Page } from "@playwright/test";

import { importGallery } from "../src/i18n/import";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { openGallery } from "./galleryPage";

/**
 * The validation tab in the gallery, on fixed data (DEC-037.1): with
 * problems, with the vault locked and without problems.
 */

/** Where the screenshots of the validation go: outside `src/`, ignored by git. */
const SHOTS = "e2e-screenshots/validation";

async function specimen(page: Page, caption: string): Promise<Locator> {
  await openGallery(page);
  const figure = page.getByRole("figure", { name: caption });
  await figure.scrollIntoViewIfNeeded();
  return figure;
}

async function shoot(figure: Locator, name: string): Promise<void> {
  await figure.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}

function rows(figure: Locator): Locator {
  return figure
    .getByRole("list", { name: texts.validation.list, exact: true })
    .getByRole("listitem");
}

test.describe("the validation tab", () => {
  test("counts the problems per level, errors first", async ({ page }) => {
    const figure = await specimen(page, importGallery.validation);

    await expect(figure.getByRole("heading", { name: "3 problemi nel documento" })).toBeVisible();
    await expect(figure.getByRole("list", { name: texts.validation.counts })).toHaveText(
      /1 errore\s*1 avviso\s*1 nota/,
    );
    await expect(rows(figure).first()).toContainText(texts.components.severity.error);
    await shoot(figure, "problems");
  });

  test("leads from every row to its node, the error included", async ({ page }) => {
    const figure = await specimen(page, importGallery.validation);

    await rows(figure).first().getByRole("button").click();

    await expect(page.locator("section#importValidation").getByRole("status").last()).toHaveText(
      "Vai al nodo: Archivio / old-gw",
    );
  });

  test("shows the note of the locked vault without a way to a node", async ({ page }) => {
    const figure = await specimen(page, importGallery.validationLocked);

    const note = rows(figure).filter({ hasText: "il vault è bloccato" });
    await expect(note).toHaveCount(1);
    await expect(note.getByRole("button")).toHaveCount(0);
    await shoot(figure, "vault-locked");
  });

  test("says when the document has no problem", async ({ page }) => {
    const figure = await specimen(page, importGallery.validationClean);

    await expect(figure.getByRole("heading", { name: texts.validation.none })).toBeVisible();
    await shoot(figure, "no-problems");
  });
});
