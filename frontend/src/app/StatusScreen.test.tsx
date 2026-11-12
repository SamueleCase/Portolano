import { screen } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import { texts } from "../i18n/it";
import { initialFakeState, installFakeBackend, rejectWith } from "../test/fakeBackend";
import { renderWithQueries } from "../test/render";
import { StatusScreen } from "./StatusScreen";

describe("StatusScreen", () => {
  it("shows the default document and the vault from the backend", async () => {
    const state = initialFakeState();
    state.defaultDocumentPath = "/Users/utente-di-prova/infra/portolano.json";
    state.vault.kind = "file";
    installFakeBackend(state);

    renderWithQueries(<StatusScreen />);

    expect(
      await screen.findByText("/Users/utente-di-prova/infra/portolano.json"),
    ).toBeInTheDocument();
    expect(screen.getByText(texts.status.documentPresent)).toBeInTheDocument();
    expect(screen.getByText(texts.status.vaultKinds.file)).toBeInTheDocument();
  });

  it("says when the default document does not exist yet", async () => {
    installFakeBackend();

    renderWithQueries(<StatusScreen />);

    expect(await screen.findByText(texts.status.documentAbsent)).toBeInTheDocument();
  });

  it("shows the message of a failed command as an alert", async () => {
    mockIPC(() => rejectWith({ code: "invalid", message: "configurazione non valida" }));

    renderWithQueries(<StatusScreen />);

    expect(await screen.findByRole("alert")).toHaveTextContent("configurazione non valida");
  });
});
