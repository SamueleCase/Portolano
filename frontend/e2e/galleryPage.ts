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

/**
 * Makes the viewport `height` tall, so that the background, fixed to the
 * viewport, covers everything the next screenshot captures, and waits for
 * the canvas to be painted again at that size.
 */
export async function fitViewport(page: Page, height: number): Promise<void> {
  await page.setViewportSize({ width: 1280, height: Math.max(800, Math.ceil(height)) });
  // Painted again at the new size: the canvas fills the viewport and its
  // bitmap has the same scale on both axes (before the repaint the height
  // of the bitmap is still the old one).
  await page.waitForFunction(() => {
    const canvas = document.querySelector("canvas");
    if (!canvas || canvas.clientHeight !== window.innerHeight) return false;
    const scaleX = canvas.width / canvas.clientWidth;
    const scaleY = canvas.height / canvas.clientHeight;
    return Math.abs(scaleX - scaleY) < 0.01;
  });
}
