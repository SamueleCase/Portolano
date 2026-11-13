import type { Locator, Page } from "@playwright/test";

import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { sessionsGallery } from "../src/i18n/sessions";
import { expect, test } from "./fixtures";
import { openGallery } from "./galleryPage";

/** Where the screenshots of the session tabs go: outside `src/`, ignored by git. */
const SHOTS = "e2e-screenshots/sessions";

const { actions, external, rdp, status } = texts.sessions;

/** The sessions section of the gallery, on the fake transport. */
async function sessions(page: Page): Promise<Locator> {
  await openGallery(page);
  const section = page.locator("section#sessions");
  await section.scrollIntoViewIfNeeded();
  await expect(section.getByRole("tab")).toHaveCount(3);
  return section;
}

/** The state shown in the header of the selected tab. */
function state(section: Locator): Locator {
  return section.getByRole("tabpanel").getByRole("status");
}

/** The terminal of the selected tab, named after its connection. */
function terminal(section: Locator): Locator {
  const name = new RegExp(`^${texts.sessions.terminal.replace("{summary}", "")}`);
  return section.getByRole("tabpanel").getByRole("group", { name });
}

async function shoot(section: Locator, name: string): Promise<void> {
  await section.screenshot({ path: `${SHOTS}/${name}.png`, animations: "disabled" });
}

test.describe("the session tabs", () => {
  test("open with the output of the terminal and the connected state", async ({ page }) => {
    const section = await sessions(page);

    await expect(terminal(section)).toContainText("Last login");
    await expect(state(section)).toHaveText(status.connected);
    await shoot(section, "ssh");
  });

  test("end with the exit code of the program", async ({ page }) => {
    const section = await sessions(page);
    await expect(terminal(section)).toContainText("operatore@web01");

    await terminal(section).click();
    await page.keyboard.type("exit 3");
    await page.keyboard.press("Enter");

    await expect(state(section)).toHaveText(interpolate(status.endedWithCode, { code: 3 }));
    await expect(section.getByRole("button", { name: actions.close })).toBeVisible();
  });

  test("show a refused connection with code 255 and bring it back with Riprova", async ({
    page,
  }) => {
    const section = await sessions(page);

    await section.getByRole("button", { name: sessionsGallery.open.refused }).click();

    await expect(state(section)).toHaveText(interpolate(status.failedWithCode, { code: 255 }));
    await expect(terminal(section)).toContainText("Connection refused");
    await shoot(section, "ssh-failed");
    await section.getByRole("button", { name: actions.retry }).click();

    // The new session took the place of the old tab, and is still selected.
    const selected = section.getByRole("tab", { selected: true });
    await expect(selected).toContainText(sessionsGallery.names["node-ssh-refused"]);
    await expect(section.getByRole("tab")).toHaveCount(4);
    await expect(state(section)).toHaveText(status.connected);
    await expect(terminal(section)).toContainText("Last login");
  });

  test("show FreeRDP in its own window with a running duration", async ({ page }) => {
    const section = await sessions(page);

    await section.getByRole("tab", { name: /dc01/ }).click();

    await expect(section.getByRole("heading", { name: rdp.title })).toBeVisible();
    await expect(state(section)).toHaveText(status.rdpRunning);
    await expect(section.getByText(/^\d\d:\d\d:\d\d$/)).toBeVisible();
    await expect(section.getByRole("button", { name: actions.disconnect })).toBeVisible();
    await shoot(section, "rdp");
  });

  test("show the log of a failed FreeRDP as text", async ({ page }) => {
    const section = await sessions(page);

    await section.getByRole("button", { name: sessionsGallery.open.rdpFailed }).click();
    await section.getByRole("button", { name: actions.showLog }).click();

    await expect(section.getByLabel(rdp.log)).toContainText("rifiutata \\u{1b}[31m");
    await shoot(section, "rdp-failed");
  });

  test("show a connection handed to Windows App", async ({ page }) => {
    const section = await sessions(page);

    await section.getByRole("tab", { name: /ts01/ }).click();

    await expect(section.getByRole("heading", { name: external.title })).toBeVisible();
    await expect(state(section)).toHaveText(status.externalOpen);
    await shoot(section, "windows-app");
  });

  test("close a tab", async ({ page }) => {
    const section = await sessions(page);

    await section.getByRole("tab", { name: /ts01/ }).click();
    await section.getByRole("button", { name: actions.close }).click();

    await expect(section.getByRole("tab")).toHaveCount(2);
  });
});
