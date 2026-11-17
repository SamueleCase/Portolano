import { editorGallery } from "../src/i18n/editor";
import { texts } from "../src/i18n/it";
import { editorSection, openFromSection, shoot } from "./editorPage";
import { expect, test } from "./fixtures";

const t = texts.preferences;

test.describe("the preferences", () => {
  test("apply at once and show the fields a choice needs", async ({ page }) => {
    const section = await editorSection(page);
    const dialog = await openFromSection(page, section, editorGallery.open.preferences, t.title);
    await expect(dialog.getByLabel(t.vault.file)).toBeVisible();
    await shoot(dialog, "preferences");

    await dialog
      .getByRole("radiogroup", { name: t.terminal.label })
      .getByRole("radio", { name: t.terminal.apps.custom })
      .click();
    await expect(dialog.getByLabel(t.terminal.command)).toBeVisible();
    await dialog
      .getByRole("radiogroup", { name: t.appearance.density })
      .getByRole("radio", { name: t.appearance.densities.compact })
      .click();

    await expect(
      dialog
        .getByRole("radiogroup", { name: t.appearance.density })
        .getByRole("radio", { name: t.appearance.densities.compact }),
    ).toHaveAttribute("aria-checked", "true");
    await shoot(dialog, "preferences-custom-terminal");
  });
});
