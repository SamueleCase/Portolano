import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { UNREADABLE_IMPORT_FILE } from "../../test/fakeBackendImport";
import {
  chooseFile,
  importBackend,
  passwordField,
  renderWizard,
  reportShown,
  toOptions,
  toPasswords,
} from "./testWizard";

/** Steps 1–3 of the import wizard on the fake backend (inventario §5.13). */

// jsdom does not lay out, so it has no `scrollIntoView`, which the folder picker calls.
beforeAll(() => {
  Element.prototype.scrollIntoView = () => undefined;
});
afterAll(() => {
  Reflect.deleteProperty(Element.prototype, "scrollIntoView");
});

describe("ImportWizard", () => {
  it("reads the chosen file at once and says it is readable", async () => {
    importBackend();
    const { user } = renderWizard();

    await chooseFile(user);

    expect(screen.getByText("9 elementi letti")).toBeInTheDocument();
    expect(screen.getByRole("dialog")).toHaveTextContent("ufficio.rtsz");
  });

  it("shows the read error of an unreadable file and stays on the file", async () => {
    importBackend({ file: UNREADABLE_IMPORT_FILE });
    const { user } = renderWizard();

    await user.click(screen.getByRole("button", { name: "Scegli file…" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Permission denied");
    expect(screen.getByRole("button", { name: "Avanti" })).toHaveAttribute("aria-disabled", "true");
  });

  it("stays on the file when the native dialog is cancelled", async () => {
    importBackend({ file: null });
    const { user } = renderWizard();

    await user.click(screen.getByRole("button", { name: "Scegli file…" }));

    expect(screen.getByRole("button", { name: "Avanti" })).toHaveAttribute("aria-disabled", "true");
  });

  it("tells in the options how many passwords and trashed items the file has", async () => {
    importBackend();
    const { user } = renderWizard();

    await toOptions(user);

    expect(
      screen.getByText("3 campi password da reinserire nel passo successivo"),
    ).toBeInTheDocument();
    expect(screen.getByText("2 elementi eliminati in Royal TSX")).toBeInTheDocument();
    expect(screen.getByText("In prova: 7 nodi nuovi, 0 aggiornati.")).toBeInTheDocument();
  });

  it("imports into the folder chosen in the picker", async () => {
    const state = importBackend();
    const { user } = renderWizard();
    await toOptions(user);

    await user.click(screen.getByRole("button", { name: "Scegli…" }));
    await user.click(await screen.findByRole("option", { name: /Rete/ }));
    await user.click(screen.getByRole("button", { name: "Scegli Rete" }));
    await screen.findByText(/^In prova:/);
    await user.click(screen.getByRole("switch", { name: "Chiedi le password" }));
    await user.click(screen.getByRole("button", { name: "Avanti" }));
    await reportShown();

    const rete = state.document?.nodes.find((node) => node.name === "Rete");
    expect(rete?.children.map((node) => node.name)).toContain("Ufficio");
  });

  it("shows the conflicts of a file imported before on the options", async () => {
    importBackend();
    const first = renderWizard();
    await toPasswords(first.user);
    await first.user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
    await reportShown();
    cleanup();
    const { user } = renderWizard();

    await chooseFile(user);
    await user.click(screen.getByRole("button", { name: "Avanti" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("esistono già nel documento");
  });

  it("asks the passwords one at a time with the node and the user", async () => {
    importBackend();
    const { user } = renderWizard();
    await toPasswords(user);

    await user.type(passwordField(), "prima-password-finta");
    await user.click(screen.getByRole("button", { name: "Salva e continua" }));

    expect(await screen.findByText("2 / 3")).toBeInTheDocument();
    expect(screen.getByText("1 inserita · 0 saltate")).toBeInTheDocument();
    expect(screen.getByText("Ufficio/Produzione/db10")).toBeInTheDocument();
    expect(screen.getByText("dba")).toBeInTheDocument();
  });

  it("empties the password field for the next request", async () => {
    importBackend();
    const { user } = renderWizard();
    await toPasswords(user);

    await user.type(passwordField(), "prima-password-finta{Enter}");

    await screen.findByText("2 / 3");
    expect(passwordField()).toHaveValue("");
  });

  it("counts a skipped password", async () => {
    importBackend();
    const { user } = renderWizard();
    await toPasswords(user);

    await user.click(screen.getByRole("button", { name: "Salta" }));

    expect(await screen.findByText("0 inserite · 1 saltata")).toBeInTheDocument();
  });

  it("imports at once with the remaining passwords skipped", async () => {
    importBackend();
    const { user } = renderWizard();
    await toPasswords(user);

    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));

    await reportShown();
    expect(
      screen.getByText("3 password saltate: i nodi risultano con password assente."),
    ).toBeInTheDocument();
  });

  it("skips the passwords step when the passwords are not asked", async () => {
    importBackend();
    const { user } = renderWizard();
    await toOptions(user);

    await user.click(screen.getByRole("switch", { name: "Chiedi le password" }));
    await user.click(screen.getByRole("button", { name: "Avanti" }));

    await reportShown();
    const steps = screen.getByRole("list", { name: "Passi" });
    expect(within(steps).getByText(/Password/)).toHaveTextContent("(saltato)");
  });

  it("asks to unlock the vault before typing passwords it cannot write", async () => {
    importBackend({ vaultLocked: true });
    const { user, onUnlockVault } = renderWizard();
    await toOptions(user);

    await user.click(screen.getByRole("button", { name: "Sblocca il vault…" }));

    expect(onUnlockVault).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Avanti" })).toHaveAttribute("aria-disabled", "true");
  });

  it("goes back to the options with the error when the import fails", async () => {
    const state = importBackend();
    const { user } = renderWizard();
    await toPasswords(user);
    state.vault.locked = true;

    await user.type(passwordField(), "prima-password-finta{Enter}");
    await screen.findByText("2 / 3");
    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("il vault è bloccato");
    await waitFor(() => expect(screen.getByText(/^In prova:/)).toBeInTheDocument());
  });
});
