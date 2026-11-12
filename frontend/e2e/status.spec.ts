import { texts } from "../src/i18n/it";
import { initialFakeState } from "../src/test/fakeBackend";
import { expect, test } from "./fixtures";

test("the status screen shows what the backend knows at startup", async ({ page }) => {
  const { defaultDocumentPath, vault } = initialFakeState();

  await page.goto("/");

  await expect(page.getByRole("heading", { name: texts.status.title })).toBeVisible();
  await expect(page.getByText(defaultDocumentPath)).toBeVisible();
  await expect(page.getByText(texts.status.vaultKinds[vault.kind])).toBeVisible();
  // The screenshot waits for the fonts itself, but only after its timeout has started.
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
  await page.screenshot({
    path: "e2e-screenshots/status-screen.png",
    fullPage: true,
    animations: "disabled",
  });
});

test("the status screen uses the fonts shipped with the app", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: texts.status.title })).toBeVisible();

  const loaded = await page.evaluate(async () => {
    await document.fonts.ready;
    return [...document.fonts]
      .filter((face) => face.status === "loaded")
      .map((face) => face.family.replaceAll(/["']/g, ""));
  });

  expect(loaded).toEqual(expect.arrayContaining(["Hanken Grotesk", "IBM Plex Mono"]));
});
