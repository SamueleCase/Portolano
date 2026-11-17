import type { QueryClient } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  askPrivateKeyFirst,
  importBackend,
  keyField,
  passwordField,
  renderWizard,
  reportShown,
  toPasswords,
} from "./testWizard";

/**
 * A private key asked by the import goes in the masked area of the secret
 * dialog, with its line breaks (DEC-044.1), and follows the rule of the
 * passwords of the wizard (DEC-037.4), checked with a sentinel secret.
 */

const SENTINEL = "sentinel-import-key-3b9c";
const KEY = `-----BEGIN OPENSSH PRIVATE KEY-----\n${SENTINEL}\n-----END OPENSSH PRIVATE KEY-----`;

/** Types `KEY` line by line: Enter in the area starts a new line. */
function typedKey(): string {
  return KEY.split("\n").join("{Enter}");
}

/** Everything the cache of Query holds, as text. */
function cacheText(client: QueryClient): string {
  return JSON.stringify([
    client
      .getMutationCache()
      .getAll()
      .map((mutation) => mutation.state),
    client
      .getQueryCache()
      .getAll()
      .map((query) => query.state),
  ]);
}

/** The page as text, with the values of its fields. */
function pageText(): string {
  const fields = [
    ...document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>("input, textarea"),
  ].map((field) => field.value);
  return [document.body.innerHTML, ...fields].join();
}

/** The wizard on the first request, which asks for a private key. */
async function toKey() {
  importBackend();
  const applied = askPrivateKeyFirst();
  const wizard = renderWizard();
  await toPasswords(wizard.user);
  return { ...wizard, applied };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("private key of the import", () => {
  it("is asked in a masked area of several lines", async () => {
    await toKey();

    expect(keyField()).toHaveAttribute("autocomplete", "off");
    expect(keyField()).toHaveAttribute("spellcheck", "false");
    expect(keyField()).toHaveFocus();
  });

  it("leaves the other requests to a password field of one line", async () => {
    const { user } = await toKey();

    await user.click(screen.getByRole("button", { name: "Salta" }));
    await screen.findByText("2 / 3");

    expect(passwordField()).toHaveAttribute("type", "password");
    expect(screen.queryByLabelText("Chiave privata")).toBeNull();
  });

  it("takes Enter as a new line of the key, not as going on", async () => {
    const { user } = await toKey();

    await user.type(keyField(), "riga 1{Enter}riga 2");

    expect(screen.getByText("1 / 3")).toBeInTheDocument();
    expect(keyField()).toHaveValue("riga 1\nriga 2");
  });

  it("reaches import_apply with its line breaks, in the order of the requests", async () => {
    const { user, applied } = await toKey();
    await user.type(keyField(), typedKey());

    await user.click(screen.getByRole("button", { name: "Salva e continua" }));
    await screen.findByText("2 / 3");
    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
    await reportShown();

    expect(applied).toEqual([
      [
        { nodePath: "Ufficio/Produzione/web10", field: { kind: "privateKey" }, value: KEY },
        { nodePath: "Ufficio/Produzione/db10", field: { kind: "password" }, value: null },
        { nodePath: "Ufficio/Admin ufficio", field: { kind: "password" }, value: null },
      ],
    ]);
  });

  it("leaves the area as soon as the user goes on", async () => {
    const { user } = await toKey();
    await user.type(keyField(), typedKey());

    await user.click(screen.getByRole("button", { name: "Salva e continua" }));
    await screen.findByText("2 / 3");

    expect(pageText()).not.toContain(SENTINEL);
  });

  it("stays out of the cache and of the page once sent", async () => {
    const { user, client } = await toKey();
    await user.type(keyField(), typedKey());

    await user.click(screen.getByRole("button", { name: "Salva e continua" }));
    await user.click(await screen.findByRole("button", { name: "Salta tutte le restanti" }));
    await reportShown();

    expect([cacheText(client), pageText()].join()).not.toContain(SENTINEL);
  });

  it("does not come back after the wizard closes on the key", async () => {
    const { user, applied, setOpen } = await toKey();
    await user.type(keyField(), typedKey());
    await user.keyboard("{Escape}");
    setOpen(false);

    setOpen(true);
    await toPasswords(user);

    expect(keyField()).toHaveValue("");
    expect(applied).toEqual([]);
    expect(pageText()).not.toContain(SENTINEL);
  });
});
