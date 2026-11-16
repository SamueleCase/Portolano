import { editorGallery } from "../src/i18n/editor";
import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { TEST_MASTER_PASSWORD } from "../src/test/fakeBackendModel";
import { editorSection, openFromSection, shoot } from "./editorPage";
import { expect, test } from "./fixtures";

const d = texts.dialogs;
const open = editorGallery.open;

test.describe("the dialogs", () => {
  test("unlock the vault, refusing a wrong master password", async ({ page }) => {
    const section = await editorSection(page);
    const dialog = await openFromSection(page, section, open.unlock, d.unlock.title);
    await shoot(dialog, "unlock");

    await dialog.getByLabel(d.unlock.field).fill("sbagliata-di-prova");
    await dialog.getByRole("button", { name: d.unlock.unlock }).click();
    await expect(dialog.getByRole("alert")).toContainText("master password errata");
    await shoot(dialog, "unlock-wrong-password");
    await dialog.getByLabel(d.unlock.field).fill(TEST_MASTER_PASSWORD);
    await dialog.getByRole("button", { name: d.unlock.unlock }).click();

    await expect(dialog).toBeHidden();
  });

  test("ask for a new master password of at least 12 characters", async ({ page }) => {
    const section = await editorSection(page);
    const dialog = await openFromSection(page, section, open.create, d.create.title);

    await dialog.getByLabel(d.create.field).fill("corta");
    await dialog.getByLabel(d.create.confirm).fill("corta");
    await dialog.getByRole("button", { name: d.create.create }).click();

    await expect(dialog.getByRole("alert")).toHaveText(d.create.tooShort);
    await shoot(dialog, "create-vault");
  });

  test("set a password after unlocking the vault", async ({ page }) => {
    const section = await editorSection(page);
    const unlock = await openFromSection(page, section, open.password, d.unlock.title);
    await unlock.getByLabel(d.unlock.field).fill(TEST_MASTER_PASSWORD);
    await unlock.getByRole("button", { name: d.unlock.unlock }).click();

    const dialog = page.getByRole("dialog", { name: d.secret.replaceTitles.password });
    await dialog.getByLabel(d.secret.fields.password).fill("una-password-di-prova");
    await dialog.getByLabel(d.secret.confirm).fill("una-password-di-prova");
    await expect(dialog.getByText(d.secret.match.password)).toBeVisible();
    await shoot(dialog, "password");
    await dialog.getByRole("button", { name: d.secret.save.file }).click();

    await expect(dialog).toBeHidden();
  });

  test("ask the credentials at launch and hand them over", async ({ page }) => {
    const section = await editorSection(page);
    const dialog = await openFromSection(page, section, open.login, d.login.title);
    await shoot(dialog, "login");

    await dialog.getByLabel(d.login.user).fill("operatore");
    await dialog.getByLabel(d.login.password).fill("una-password-di-prova");
    await dialog.getByRole("button", { name: d.login.connect }).click();

    await expect(section.getByRole("status")).toHaveText(
      interpolate(editorGallery.login, { user: "operatore" }),
    );
  });

  test("show where a credential comes from in each mode", async ({ page }) => {
    const section = await editorSection(page);
    const modes = texts.components.credentialModes;
    const dialog = await openFromSection(
      page,
      section,
      open.credential,
      texts.components.credentialSelector.label,
    );
    await expect(dialog.getByText("Admin ACME").first()).toBeVisible();
    await shoot(dialog, "credential-inherit");

    await dialog.getByRole("radio", { name: new RegExp(`^${modes.byName.title}`) }).click();
    await dialog.getByLabel(texts.components.credentialPreview.byNameField).fill("inesistente");
    await expect(dialog.getByText(texts.components.credentialPreview.byNameNotFound)).toBeVisible();
    await shoot(dialog, "credential-by-name-not-found");
    await dialog.getByLabel(texts.components.credentialPreview.byNameField).fill("deploy");
    await expect(dialog.getByText("Rete / deploy")).toBeVisible();
    await shoot(dialog, "credential-by-name");
  });

  test("choose an icon and a jump host", async ({ page }) => {
    const section = await editorSection(page);
    const icons = await openFromSection(
      page,
      section,
      open.icon,
      texts.components.iconPicker.title,
    );
    await icons.getByRole("radio", { name: texts.icons.database }).click();
    await shoot(icons, "icon-picker");
    await icons.getByRole("button", { name: texts.components.iconPicker.use }).click();
    await expect(icons).toBeHidden();

    const jumps = await openFromSection(page, section, open.jumpHost, d.picker.jumpHost.title);
    await expect(jumps.getByText(/web01 è escluso/)).toBeVisible();
    await shoot(jumps, "jump-host-picker");
  });
});
