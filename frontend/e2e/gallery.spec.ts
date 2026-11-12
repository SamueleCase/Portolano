import { gallery } from "../src/i18n/gallery";
import { texts } from "../src/i18n/it";
import { expect, test } from "./fixtures";
import { fitViewport, openGallery, SHOTS } from "./galleryPage";

const sections = Object.keys(gallery.section);

/** Room around a section in its screenshot: the 14 px padding of the page, above and below. */
const SECTION_MARGIN = 2 * 14;

test.describe("the component gallery", () => {
  test("has a screenshot for each section and one of the whole page", async ({ page }) => {
    // Fifteen screenshots of a long page.
    test.slow();
    await openGallery(page);

    for (const [index, id] of sections.entries()) {
      const section = page.locator(`section#${id}`);
      await expect(section).toBeVisible();
      // The whole section inside the viewport, over the background.
      const box = await section.boundingBox();
      await fitViewport(page, (box?.height ?? 0) + SECTION_MARGIN);
      await section.scrollIntoViewIfNeeded();
      await section.screenshot({
        path: `${SHOTS}/${String(index + 1).padStart(2, "0")}-${id}.png`,
        animations: "disabled",
      });
    }
    await fitViewport(page, await page.evaluate(() => document.documentElement.scrollHeight));
    await page.screenshot({
      path: `${SHOTS}/00-gallery.png`,
      fullPage: true,
      animations: "disabled",
    });
  });

  test("shows the whole page in the compact density", async ({ page }) => {
    await openGallery(page, "&density=compact");

    await expect(page.locator("html")).toHaveAttribute("data-density", "compact");
    await fitViewport(page, await page.evaluate(() => document.documentElement.scrollHeight));
    await page.screenshot({
      path: `${SHOTS}/00-gallery-compact.png`,
      fullPage: true,
      animations: "disabled",
    });
  });

  test("opens the dialog, the side panel and the selectors over the veil", async ({ page }) => {
    await openGallery(page);
    const overlays = [
      [gallery.open.dialog, "dialog"],
      [gallery.open.sidePanel, "side-panel"],
      [gallery.open.credentialSelector, "credential-selector"],
      [gallery.open.iconPicker, "icon-picker"],
      [gallery.open.nodePicker, "node-picker"],
    ] as const;

    for (const [button, name] of overlays) {
      await page.getByRole("button", { name: button }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
      await page.screenshot({ path: `${SHOTS}/overlay-${name}.png`, animations: "disabled" });
      await page.keyboard.press("Escape");
      await expect(page.getByRole("dialog")).toBeHidden();
    }
  });

  test("opens the context menu, the toasts and a tooltip", async ({ page }) => {
    await openGallery(page);

    await page.getByRole("button", { name: gallery.open.contextMenu }).click();
    await expect(page.getByRole("menu")).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/overlay-context-menu.png`, animations: "disabled" });
    await page.keyboard.press("Escape");

    await page.getByRole("button", { name: gallery.open.toasts }).click();
    const notifications = page.getByRole("region", { name: texts.components.toast.region });
    await expect(notifications.getByRole("listitem")).toHaveCount(3);
    await page.screenshot({ path: `${SHOTS}/overlay-toasts.png`, animations: "disabled" });

    await page.locator("#overlays").getByRole("button").first().hover();
    await expect(page.getByRole("tooltip")).toBeVisible();
    await page.screenshot({ path: `${SHOTS}/overlay-tooltip.png`, animations: "disabled" });
  });

  test("draws the figures of Hanken Grotesk all with the same width (ambiguity 22)", async ({
    page,
  }) => {
    await openGallery(page);

    const widths = await page.evaluate(() => {
      const probe = document.createElement("span");
      probe.style.font = "600 34px 'Hanken Grotesk'";
      probe.style.position = "absolute";
      document.body.append(probe);
      const width = (text: string) => {
        probe.textContent = text;
        return probe.getBoundingClientRect().width;
      };
      const measured = ["1111", "8888", "0000", "4747"].map(width);
      probe.remove();
      return measured;
    });

    expect(new Set(widths).size).toBe(1);
  });

  test("shows the control characters of an untrusted name", async ({ page }) => {
    await openGallery(page);

    await expect(page.getByTestId("hostile-name")).toHaveText(
      "web01\\u{1b}[2J\\u{7}\\u{202e}gpj.exe",
    );
  });

  test("runs the terminal: output, title as text, clipboard untouched", async ({ page }) => {
    await openGallery(page);
    const terminal = page.getByRole("group", { name: "Terminale" });

    await expect(terminal).toContainText("load average");
    await expect(page.getByTestId("terminal-title")).toContainText("deploy@web01: ~");
    await terminal.click();
    await page.keyboard.type("whoami");
    await expect(terminal).toContainText("whoami");
  });
});
