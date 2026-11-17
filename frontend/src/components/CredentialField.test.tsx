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

  it("reads the message it is given with Cambia", () => {
    render(
      <>
        <CredentialField
          label="Credenziale"
          mode="Eredita"
          backend="file"
          onChange={vi.fn()}
          describedBy="credenziale-errore"
        />
        <span id="credenziale-errore">credenziale rifiutata</span>
      </>,
    );

    expect(screen.getByRole("button", { name: "Cambia" })).toHaveAccessibleDescription(
      "credenziale rifiutata",
    );
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

  it("reads the message it is given with its actions", () => {
    render(
      <>
        <SecretRow
          label="Password"
          state="present"
          backend="file"
          onSet={vi.fn()}
          onClear={vi.fn()}
          describedBy="password-errore"
        />
        <span id="password-errore">password rifiutata</span>
      </>,
    );

    expect(screen.getByRole("button", { name: "Sostituisci…" })).toHaveAccessibleDescription(
      "password rifiutata",
    );
    expect(screen.getByRole("button", { name: "Cancella" })).toHaveAccessibleDescription(
      "password rifiutata",
    );
  });
});

describe("SecretRow with a property of the document", () => {
  it("shows control characters of the name of the property", () => {
    render(<SecretRow label={"Token\u001b API"} state="absent" backend="file" onSet={vi.fn()} />);

    expect(screen.getByRole("group", { name: "Token\\u{1b} API" })).toBeInTheDocument();
  });
});
