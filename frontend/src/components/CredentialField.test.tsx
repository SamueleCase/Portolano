import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { CredentialField } from "./CredentialField";
import { SecretRow } from "./SecretRow";

describe("CredentialField", () => {
  it("opens the selector with Cambia", async () => {
    const onChange = vi.fn();
    render(
      <CredentialField
        label="Credenziale"
        mode="Eredita"
        origin="dalla cartella che contiene il nodo"
        password="present"
        backend="keychain"
        onChange={onChange}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Cambia" }));

    expect(onChange).toHaveBeenCalledOnce();
    expect(screen.getByRole("group", { name: "Credenziale" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Password presente" })).toBeInTheDocument();
  });

  it("leads to the credential from the missing password", async () => {
    const onShowCredential = vi.fn();
    render(
      <CredentialField
        label="Credenziale del gateway"
        mode="Da una credenziale"
        origin="admin · ACME"
        password="absent"
        backend="keychain"
        onChange={vi.fn()}
        onShowCredential={onShowCredential}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "password assente" }));

    expect(onShowCredential).toHaveBeenCalledOnce();
  });
});

describe("SecretRow", () => {
  it("offers to replace and clear a present secret", async () => {
    const onClear = vi.fn();
    render(
      <SecretRow
        label="Password"
        state="present"
        backend="keychain"
        onSet={vi.fn()}
        onClear={onClear}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Cancella" }));

    expect(screen.getByRole("button", { name: "Sostituisci…" })).toBeInTheDocument();
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("offers to set an absent secret", () => {
    render(<SecretRow label="Passphrase" state="absent" backend="file" onSet={vi.fn()} />);

    expect(screen.getByRole("button", { name: "Imposta…" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Cancella" })).not.toBeInTheDocument();
  });

  it("cannot set a secret while the vault is locked", async () => {
    const onSet = vi.fn();
    render(<SecretRow label="Password" state="locked" backend="file" onSet={onSet} />);

    await userEvent.click(screen.getByRole("button", { name: "Imposta…" }));

    expect(onSet).not.toHaveBeenCalled();
  });
});
