import type { Page } from "@playwright/test";

import { texts } from "../src/i18n/it";
import type { FakeSeed } from "../src/test/fakeBackend";
import { expect, test } from "./fixtures";

const SHOTS = "e2e-screenshots/main-window";

/** The app without a document: the fake backend starts as `seed` says. */
async function firstRun(page: Page, seed: FakeSeed): Promise<void> {
  await page.addInitScript((value) => {
    window.__portolanoFakeSeed = value;
  }, seed);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

test.describe("the first run", () => {
  test("appears when no document is open", async ({ page }) => {
    await firstRun(page, { document: "none" });

    await expect(page.getByText(texts.firstRun.lead)).toBeVisible();
    await expect(
      page.getByRole("button", { name: new RegExp(texts.firstRun.create.title) }),
    ).toBeVisible();
    await expect(page.getByText(texts.firstRun.open.text)).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/first-run.png`, animations: "disabled" });
  });

  test("creates a new document where the native dialog says and opens it empty", async ({
    page,
  }) => {
    await firstRun(page, {
      document: "none",
      dialogs: { newDocument: "/Users/utente-di-prova/infra/nuovo.json" },
    });

    await page.getByRole("button", { name: new RegExp(texts.firstRun.create.title) }).click();

    await expect(page.getByRole("heading", { name: texts.firstRun.empty.title })).toBeVisible();
    await expect(page.getByText("nuovo.json")).toBeVisible();
    await expect(page.getByText(texts.firstRun.empty.text.file)).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/empty-document.png`, animations: "disabled" });
  });

  test("stays on the first run when the native dialog is cancelled", async ({ page }) => {
    await firstRun(page, { document: "none" });

    await page.getByRole("button", { name: new RegExp(texts.firstRun.create.title) }).click();

    await expect(page.getByText(texts.firstRun.lead)).toBeVisible();
  });

  test("opens an existing document chosen with the native dialog", async ({ page }) => {
    await firstRun(page, {
      document: "none",
      dialogs: { document: "/Users/utente-di-prova/infra/portolano.json" },
    });

    await page.getByRole("button", { name: new RegExp(texts.firstRun.open.title) }).click();

    await expect(page.getByRole("treeitem", { name: "Clienti" })).toBeVisible();
  });
});
