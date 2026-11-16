import { act, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { TEST_MASTER_PASSWORD, initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { renderWithQueries } from "../../test/render";
import { CreateVaultDialog } from "./CreateVaultDialog";
import { SetSecretDialog } from "./SetSecretDialog";
import { UnlockVaultDialog } from "./UnlockVaultDialog";
import { useVaultUnlock } from "./useVaultUnlock";

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

function VaultGate({ onResult }: { onResult: (unlocked: boolean) => void }) {
  const { ensureUnlocked, dialog } = useVaultUnlock();
  return (
    <>
      <button type="button" onClick={() => void ensureUnlocked().then(onResult)}>
        prova
      </button>
      {dialog}
    </>
  );
}

describe("useVaultUnlock", () => {
  it("answers at once when the vault is open", async () => {
    unlocked();
    const onResult = vi.fn();
    renderWithQueries(<VaultGate onResult={onResult} />);

    await userEvent.click(screen.getByRole("button", { name: "prova" }));

    await waitFor(() => expect(onResult).toHaveBeenCalledWith(true));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("asks for the master password of a locked vault", async () => {
    installFakeBackend();
    const onResult = vi.fn();
    renderWithQueries(<VaultGate onResult={onResult} />);

    await userEvent.click(screen.getByRole("button", { name: "prova" }));
    await userEvent.type(await screen.findByLabelText(t.unlock.field), TEST_MASTER_PASSWORD);
    await userEvent.click(screen.getByRole("button", { name: t.unlock.unlock }));

    await waitFor(() => expect(onResult).toHaveBeenCalledWith(true));
  });

  it("answers false when it is unmounted while asking", async () => {
    installFakeBackend();
    const onResult = vi.fn();
    const { unmount } = renderWithQueries(<VaultGate onResult={onResult} />);
    await userEvent.click(screen.getByRole("button", { name: "prova" }));
    await screen.findByLabelText(t.unlock.field);

    unmount();

    await waitFor(() => expect(onResult).toHaveBeenCalledWith(false));
  });

  it("answers false when the user gives up", async () => {
    installFakeBackend();
    const onResult = vi.fn();
    renderWithQueries(<VaultGate onResult={onResult} />);

    await userEvent.click(screen.getByRole("button", { name: "prova" }));
    await userEvent.click(await screen.findByRole("button", { name: t.unlock.withoutSecrets }));

    await waitFor(() => expect(onResult).toHaveBeenCalledWith(false));
  });
});

function SecretHost({ id, onSaved }: { id: string; onSaved: () => void }) {
  const [open, setOpen] = useState(true);
  return (
    <SetSecretDialog
      request={
        open
          ? {
              nodeId: id,
              nodePath: "/Clienti/Admin ACME",
              slot: { kind: "password" },
              replacing: true,
            }
          : null
      }
      onClose={() => setOpen(false)}
      onSaved={onSaved}
    />
  );
}

describe("SetSecretDialog", () => {
  it("asks to unlock first, then saves the password", async () => {
    const state = installFakeBackend();
    const id = await idOf("Admin ACME");
    const onSaved = vi.fn();
    renderWithQueries(<SecretHost id={id} onSaved={onSaved} />);

    await userEvent.type(await screen.findByLabelText(t.unlock.field), TEST_MASTER_PASSWORD);
    await userEvent.click(screen.getByRole("button", { name: t.unlock.unlock }));
    await userEvent.type(await screen.findByLabelText(t.secret.fields.password), "nuova");
    await userEvent.type(screen.getByLabelText(t.secret.confirm), "nuova");
    await act(() => userEvent.click(screen.getByRole("button", { name: t.secret.save.file })));

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(state.vault.locked).toBe(false);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
