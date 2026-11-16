import { editorGallery } from "../src/i18n/editor";
import { interpolate } from "../src/i18n/format";
import { texts } from "../src/i18n/it";
import { TEST_MASTER_PASSWORD } from "../src/test/fakeBackendModel";
import { editorSection, openFromSection, shoot } from "./editorPage";
import { expect, test } from "./fixtures";

const t = texts.editor;
const open = editorGallery.open;
/** A sentinel secret: it must never reach the page once staged. */
const SENTINEL = "sentinel-e2e-editor-secret-c41b";

test.describe("the editor", () => {
  test("creates an SSH connection in the folder it was opened on", async ({ page }) => {
    const section = await editorSection(page);
    const panel = await openFromSection(page, section, open.newSsh, t.subtitles.add.ssh);
    await shoot(panel, "new-ssh-general");

    await panel.getByLabel(t.general.name).fill("web09");
    await panel.getByRole("button", { name: t.next }).click();
    await expect(panel.getByLabel(t.connection.port)).toHaveAttribute("placeholder", "22");
    await shoot(panel, "new-ssh-connection");
    await panel.getByLabel(t.connection.host, { exact: true }).fill("web09.example.com");
    await panel.getByRole("button", { name: t.save }).click();

    await expect(section.getByRole("status")).toHaveText(
      interpolate(editorGallery.saved, { path: "/Clienti/ACME/web09" }),
    );
  });

  test("shows the second step of an RDP connection, a folder and a credential", async ({
    page,
  }) => {
    const section = await editorSection(page);
    for (const [label, name, shot] of [
      [open.newRdp, t.subtitles.add.rdp, "new-rdp-connection"],
      [open.newFolder, t.subtitles.add.folder, "new-folder-credential"],
      [open.newCredential, t.subtitles.add.credential, "new-credential-access"],
    ] as const) {
      const panel = await openFromSection(page, section, label, name);
      await panel.getByRole("button", { name: t.next }).click();
      await shoot(panel, shot);
      await panel.getByRole("button", { name: t.cancel }).first().click();
      await expect(panel).toBeHidden();
    }
  });

  test("shows an RDP connection with its gateway", async ({ page }) => {
    const section = await editorSection(page);
    const panel = await openFromSection(page, section, open.editRdp, /ts01/);

    await panel.getByRole("button", { name: t.next }).click();

    await expect(panel.getByLabel(t.rdp.gatewayHost)).toHaveValue("gw.example.com");
    await shoot(panel, "edit-rdp-gateway");
  });

  test("stages a password until Salva and asks to unlock the vault then", async ({ page }) => {
    const section = await editorSection(page);
    const panel = await openFromSection(page, section, open.editCredential, /deploy/);
    await panel.getByRole("button", { name: t.next }).click();
    await panel
      .getByRole("group", { name: t.credential.password })
      .getByRole("button", { name: texts.components.secretRow.set })
      .click();

    const dialog = page.getByRole("dialog", { name: texts.dialogs.secret.titles.password });
    await dialog.getByLabel(texts.dialogs.secret.fields.password).fill(SENTINEL);
    await dialog.getByLabel(texts.dialogs.secret.confirm).fill(SENTINEL);
    await shoot(dialog, "password-staged-dialog");
    await dialog.getByRole("button", { name: texts.dialogs.secret.stage }).click();
    await expect(panel.getByText(texts.components.secretBadge.staged)).toBeVisible();
    await shoot(panel, "edit-credential-staged");
    await panel.getByRole("button", { name: t.save }).click();
    const unlock = page.getByRole("dialog", { name: texts.dialogs.unlock.title });
    await unlock.getByLabel(texts.dialogs.unlock.field).fill(TEST_MASTER_PASSWORD);
    await unlock.getByRole("button", { name: texts.dialogs.unlock.unlock }).click();

    await expect(section.getByRole("status")).toHaveText(
      interpolate(editorGallery.saved, { path: "/Rete/deploy" }),
    );
    expect(await page.content()).not.toContain(SENTINEL);
  });

  test("chooses the folder of a node with the folder picker", async ({ page }) => {
    const section = await editorSection(page);
    const panel = await openFromSection(page, section, open.editSsh, /web01/);

    await panel.getByRole("button", { name: t.general.change }).click();
    const picker = page.getByRole("dialog", { name: texts.dialogs.picker.folder.title });
    await shoot(picker, "folder-picker");
    await picker.getByRole("option", { name: /Rete/ }).click();
    await picker.getByRole("button", { name: /^Scegli Rete/ }).click();
    await panel.getByRole("button", { name: t.save }).click();

    await expect(section.getByRole("status")).toHaveText(
      interpolate(editorGallery.saved, { path: "/Rete/web01" }),
    );
  });

  test("adds a jump host from the tree and a free address", async ({ page }) => {
    const section = await editorSection(page);
    const panel = await openFromSection(page, section, open.editSsh, /web01/);
    await panel.getByRole("button", { name: t.next }).click();

    await panel.getByRole("button", { name: t.ssh.freeAddress }).click();
    await panel.getByLabel(t.ssh.address).fill("ops@198.51.100.40:2222");
    await panel.getByLabel(t.ssh.address).press("Enter");

    await expect(panel.getByRole("list", { name: t.ssh.jumps })).toContainText(
      "ops@198.51.100.40:2222",
    );
    await shoot(panel, "edit-ssh-jumps");
  });
});
