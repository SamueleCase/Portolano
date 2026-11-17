import type { Locator, Page } from "@playwright/test";

import { editorGallery } from "../src/i18n/editor";
import { expect } from "./fixtures";
import { openGallery } from "./galleryPage";

/** Where the screenshots of the editor, the dialogs and the preferences go (ignored by git). */
export const EDITOR_SHOTS = "e2e-screenshots/editor";

/** The height of the window in these scenarios: the longest step of the editor fits. */
const PANEL_HEIGHT = 1200;

/** The gallery section of UI-WP5, on the fake backend, once the sample document is read. */
export async function editorSection(page: Page): Promise<Locator> {
  // Tall enough for the whole side panel of the editor in one screenshot.
  await page.setViewportSize({ width: 1280, height: PANEL_HEIGHT });
  await openGallery(page);
  const section = page.locator("section#editor");
  await section.scrollIntoViewIfNeeded();
  await expect(section.getByRole("button", { name: editorGallery.open.editSsh })).toBeVisible();
  return section;
}

/** Opens what `label` opens in the section, and the dialog or panel named `name`. */
export async function openFromSection(
  page: Page,
  section: Locator,
  label: string,
  name: string | RegExp,
): Promise<Locator> {
  const dialog = page.getByRole("dialog", { name });
  // The tree may not be read yet at the first click: try until the dialog opens.
  await expect(async () => {
    await section.getByRole("button", { name: label }).click();
    await expect(dialog).toBeVisible({ timeout: 1000 });
  }).toPass();
  return dialog;
}

/** The IPC of the page as the fake backend installs it (`mockIPC`). */
interface IpcWindow {
  __TAURI_INTERNALS__: { invoke: (...args: unknown[]) => Promise<unknown> };
  __ipcAnswers: string[];
}

/**
 * Keeps every answer of the fake backend from now on, errors included, to
 * check that none carries a secret (SPEC-UI §7).
 */
export async function recordIpcAnswers(page: Page): Promise<void> {
  await page.evaluate(() => {
    const ipc = window as unknown as IpcWindow;
    const invoke = ipc.__TAURI_INTERNALS__.invoke.bind(ipc.__TAURI_INTERNALS__);
    ipc.__ipcAnswers = [];
    ipc.__TAURI_INTERNALS__.invoke = async (...args: unknown[]) => {
      try {
        const answer = await invoke(...args);
        ipc.__ipcAnswers.push(JSON.stringify(answer) ?? "");
        return answer;
      } catch (failure) {
        ipc.__ipcAnswers.push(JSON.stringify(failure) ?? "");
        throw failure;
      }
    };
  });
}

/** The answers recorded by {@link recordIpcAnswers}, as text. */
export async function ipcAnswers(page: Page): Promise<string> {
  return page.evaluate(() => (window as unknown as IpcWindow).__ipcAnswers.join("\n"));
}

/** A screenshot of a dialog or of the side panel of the editor. */
export async function shoot(surface: Locator, name: string): Promise<void> {
  await surface.screenshot({ path: `${EDITOR_SHOTS}/${name}.png`, animations: "disabled" });
}
