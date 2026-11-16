import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { openApp, row, shoot } from "./mainWindowPage";

const create = new RegExp(texts.firstRun.create.title);

test.describe("the first run", () => {
  test("appears when no document is open", async ({ page }) => {
    await openApp(page, { document: "none" });

    await expect(page.getByText(texts.firstRun.lead)).toBeVisible();
    await expect(page.getByRole("button", { name: create })).toBeVisible();
    await expect(page.getByText(texts.firstRun.open.text)).toBeVisible();
    await shoot(page, "first-run");
  });

  test("creates a new document where the native dialog says and opens it empty", async ({
    page,
  }) => {
    await openApp(page, {
      document: "none",
      dialogs: { newDocument: "/Users/utente-di-prova/infra/nuovo.json" },
    });

    await page.getByRole("button", { name: create }).click();

    await expect(page.getByRole("heading", { name: texts.firstRun.empty.title })).toBeVisible();
    await expect(page.getByText("nuovo.json")).toBeVisible();
    await expect(page.getByText(texts.firstRun.empty.text.file)).toBeVisible();
    await shoot(page, "empty-document");
  });

  test("stays on the first run when the native dialog is cancelled", async ({ page }) => {
    await openApp(page, { document: "none" });

    await page.getByRole("button", { name: create }).click();

    await expect(page.getByText(texts.firstRun.lead)).toBeVisible();
  });

  test("opens an existing document chosen with the native dialog", async ({ page }) => {
    await openApp(page, {
      document: "none",
      dialogs: { document: "/Users/utente-di-prova/infra/portolano.json" },
    });

    await page.getByRole("button", { name: new RegExp(texts.firstRun.open.title) }).click();

    await expect(row(page, "Clienti")).toBeVisible();
  });
});
