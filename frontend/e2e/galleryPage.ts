import type { Page } from "@playwright/test";

import { gallery } from "../src/i18n/gallery";
import { expect } from "./fixtures";

/** Where the screenshots of the gallery go: outside `src/`, ignored by git. */
export const SHOTS = "e2e-screenshots/gallery";

/**
 * Opens the gallery ready for a stable screenshot: reduced motion (a single
 * still drawing of the seeded sky, fades instead of movements) and the
 * fonts loaded.
 */
export async function openGallery(page: Page, query = ""): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(`/?gallery${query}`);
  await expect(page.getByRole("heading", { name: gallery.title })).toBeVisible();
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}
