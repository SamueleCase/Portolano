import { describe, expect, it } from "vitest";

import { texts } from "../i18n/it";
import { presentError, presentFailure } from "./errors";
import { IpcFailure } from "./result";

describe("presentError", () => {
  it("puts an invalid value under the field the backend names", () => {
    const presented = presentError({
      code: "invalid",
      message: "esiste già un nodo chiamato 'web01'",
      details: { kind: "invalid", field: "name" },
    });

    expect(presented).toEqual({
      presentation: "field",
      title: texts.ipc.errorTitles.invalid,
      message: "esiste già un nodo chiamato 'web01'",
      field: "name",
      details: { kind: "invalid", field: "name" },
    });
  });

  it("asks for the master password when the vault is locked", () => {
    expect(presentError({ code: "vaultLocked", message: "vault bloccato" }).presentation).toBe(
      "unlockVault",
    );
  });

  it("offers to reload a document changed by another program", () => {
    expect(presentError({ code: "modifiedExternally", message: "ricaricalo" }).presentation).toBe(
      "reloadBanner",
    );
  });

  it("shows the references that prevent a deletion in the confirmation", () => {
    const presented = presentError({
      code: "referenced",
      message: "usato da 1 riferimento",
      details: { kind: "referenced", references: [] },
    });

    expect(presented.presentation).toBe("confirmDelete");
    expect(presented.details).toEqual({ kind: "referenced", references: [] });
  });

  it("keeps the candidates of an ambiguous name", () => {
    const candidates = [{ id: "00000000-0000-4000-8000-000000000001", path: "/A/Admin" }];

    const presented = presentError({
      code: "ambiguous",
      message: "'Admin' è ambiguo",
      details: { kind: "ambiguous", candidates },
    });

    expect(presented.details).toEqual({ kind: "ambiguous", candidates });
  });
});

describe("presentFailure", () => {
  it("presents the error of a failed command", () => {
    const failure = new IpcFailure({ code: "connect", message: "ssh non trovato" });

    expect(presentFailure(failure).title).toBe(texts.ipc.errorTitles.connect);
  });

  it("presents anything else as an unexpected error, without its text", () => {
    const presented = presentFailure(new Error("dettaglio interno"));

    expect(presented).toEqual({
      presentation: "toast",
      title: texts.ipc.errorTitles.internal,
      message: texts.ipc.unexpected,
    });
  });
});
