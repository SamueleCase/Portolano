import { act, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { TEST_MASTER_PASSWORD, initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { renderWithQueries } from "../../test/render";
import { CreateVaultDialog } from "./CreateVaultDialog";
import { SetSecretDialog } from "./SetSecretDialog";
import { UnlockVaultDialog } from "./UnlockVaultDialog";
import { renderInWindow } from "../../test/renderInWindow";

const t = texts.dialogs;

describe("UnlockVaultDialog", () => {
  it("opens the vault with the master password", async () => {
    const state = installFakeBackend();
    const onUnlocked = vi.fn();
    renderWithQueries(<UnlockVaultDialog open onClose={() => undefined} onUnlocked={onUnlocked} />);

    await userEvent.type(screen.getByLabelText(t.unlock.field), TEST_MASTER_PASSWORD);
    await userEvent.click(screen.getByRole("button", { name: t.unlock.unlock }));

    await waitFor(() => expect(onUnlocked).toHaveBeenCalledOnce());
    expect(state.vault.locked).toBe(false);
  });

  it("shows a wrong master password under the field and empties it", async () => {
    installFakeBackend();
    renderWithQueries(
      <UnlockVaultDialog open onClose={() => undefined} onUnlocked={() => undefined} />,
    );

    await userEvent.type(screen.getByLabelText(t.unlock.field), "sbagliata-di-prova");
    await userEvent.click(screen.getByRole("button", { name: t.unlock.unlock }));

    expect(await screen.findByRole("alert")).toHaveTextContent("master password errata");
    expect(screen.getByLabelText(t.unlock.field)).toHaveValue("");
  });

  it("closes without unlocking with Apri senza segreti", async () => {
    installFakeBackend();
    const onClose = vi.fn();
    renderWithQueries(<UnlockVaultDialog open onClose={onClose} onUnlocked={() => undefined} />);

    await userEvent.click(screen.getByRole("button", { name: t.unlock.withoutSecrets }));

    expect(onClose).toHaveBeenCalledOnce();
  });
});

describe("CreateVaultDialog", () => {
  function missingVault() {
    const state = initialFakeState();
    state.vault.exists = false;
    return installFakeBackend(state);
  }

  it("refuses a master password shorter than 12 characters", async () => {
    missingVault();
    renderWithQueries(
      <CreateVaultDialog open onClose={() => undefined} onCreated={() => undefined} />,
    );

    await userEvent.type(screen.getByLabelText(t.create.field), "corta");
    await userEvent.type(screen.getByLabelText(t.create.confirm), "corta");
    await userEvent.click(screen.getByRole("button", { name: t.create.create }));

    expect(screen.getByRole("alert")).toHaveTextContent(t.create.tooShort);
  });

  it("creates the vault with a long enough master password", async () => {
    const state = missingVault();
    const onCreated = vi.fn();
    renderWithQueries(<CreateVaultDialog open onClose={() => undefined} onCreated={onCreated} />);

    await userEvent.type(screen.getByLabelText(t.create.field), "una-master-password-lunga");
    await userEvent.type(screen.getByLabelText(t.create.confirm), "una-master-password-lunga");
    await userEvent.click(screen.getByRole("button", { name: t.create.create }));

    await waitFor(() => expect(onCreated).toHaveBeenCalledOnce());
    expect(state.vault).toMatchObject({ exists: true, locked: false });
  });
});

describe("SetSecretDialog", () => {
  async function openFor(name: string, onClose = vi.fn()) {
    const id = await idOf(name);
    renderInWindow(
      <SetSecretDialog
        node={{ id, path: `/Clienti/${name}` }}
        field={{ kind: "password" }}
        onClose={onClose}
      />,
    );
    return onClose;
  }

  it("asks the window to unlock the vault first, then saves the password", async () => {
    const state = installFakeBackend();
    const onClose = await openFor("Admin ACME");

    await userEvent.type(await screen.findByLabelText(t.unlock.field), TEST_MASTER_PASSWORD);
    await userEvent.click(screen.getByRole("button", { name: t.unlock.unlock }));
    const dialog = await screen.findByRole("dialog", { name: t.secret.replaceTitles.password });
    await userEvent.type(within(dialog).getByLabelText(t.secret.fields.password), "nuova");
    await userEvent.type(within(dialog).getByLabelText(t.secret.confirm), "nuova");
    await act(() => userEvent.click(screen.getByRole("button", { name: t.secret.save.file })));

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(state.vault.locked).toBe(false);
  });

  it("closes without asking the password when the vault stays locked", async () => {
    installFakeBackend();
    const onClose = await openFor("Admin ACME");

    await userEvent.click(await screen.findByRole("button", { name: t.unlock.withoutSecrets }));

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(screen.queryByLabelText(t.secret.fields.password)).not.toBeInTheDocument();
  });

  it("says Imposta for a secret the node does not have yet", async () => {
    unlocked();
    const id = await idOf("deploy");
    renderInWindow(
      <SetSecretDialog
        node={{ id, path: "/Rete/deploy" }}
        field={{ kind: "password" }}
        onClose={() => undefined}
      />,
    );

    expect(
      await screen.findByRole("dialog", { name: t.secret.titles.password }),
    ).toBeInTheDocument();
  });
});
