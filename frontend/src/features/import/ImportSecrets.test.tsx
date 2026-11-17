import type { QueryClient } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  importBackend,
  passwordField,
  recordCommands,
  renderWizard,
  reportShown,
  sent,
  toOptions,
  toPasswords,
} from "./testWizard";

/**
 * The passwords typed in the wizard reach the backend only in
 * `import_apply`, and nothing else keeps them (SPEC-UI §1.2), checked with a
 * sentinel secret.
 */

const SENTINEL = "sentinel-import-secret-5e7a";

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

/** Whether the page shows or holds the sentinel anywhere. */
function pageText(): string {
  const values = [...document.querySelectorAll("input")].map((input) => input.value);
  return [document.body.innerHTML, ...values].join();
}

function carrying(commands: ReturnType<typeof recordCommands>) {
  return commands.mock.calls.filter(([, args]) => JSON.stringify(args ?? null).includes(SENTINEL));
}

/** Types the sentinel as the first password and skips the other two. */
async function importWithSentinel() {
  importBackend();
  const commands = recordCommands();
  const wizard = renderWizard();
  await toPasswords(wizard.user);
  await wizard.user.type(passwordField(), `${SENTINEL}{Enter}`);
  await screen.findByText("2 / 3");
  await wizard.user.click(screen.getByRole("button", { name: "Salta" }));
  await screen.findByText("3 / 3");
  await wizard.user.click(screen.getByRole("button", { name: "Salta" }));
  await reportShown();
  return { ...wizard, commands };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("passwords of the import", () => {
  it("reach the backend only in import_apply, in the order of the requests", async () => {
    const { commands } = await importWithSentinel();

    const calls = carrying(commands);
    expect(calls.map(([command]) => command)).toEqual(["import_apply"]);
    expect(calls[0]?.[1]).toMatchObject({
      answers: [
        { nodePath: "Ufficio/Produzione/web10", value: SENTINEL },
        { nodePath: "Ufficio/Produzione/db10", value: null },
        { nodePath: "Ufficio/Admin ufficio", value: null },
      ],
    });
  });

  it("stay out of the cache of Query", async () => {
    const { client } = await importWithSentinel();

    expect(cacheText(client)).not.toContain(SENTINEL);
  });

  it("stay out of the console", async () => {
    const logged = (["log", "info", "warn", "error", "debug"] as const).map((level) =>
      vi.spyOn(console, level),
    );

    await importWithSentinel();

    const lines = logged.flatMap((spy) => spy.mock.calls.map((call) => JSON.stringify(call)));
    expect(lines.filter((line) => line.includes(SENTINEL))).toEqual([]);
  });

  it("stay out of the page once sent", async () => {
    await importWithSentinel();

    expect(pageText()).not.toContain(SENTINEL);
  });

  it("do not come back in the import that follows a failed one", async () => {
    const state = importBackend();
    const commands = recordCommands();
    const { client, user } = renderWizard();
    await toPasswords(user);
    state.vault.locked = true;
    await user.type(passwordField(), `${SENTINEL}{Enter}`);
    await screen.findByText("2 / 3");
    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
    await screen.findByRole("alert");
    state.vault.locked = false;

    await user.click(screen.getByRole("button", { name: "Avanti" }));
    await screen.findByText("1 / 3");
    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
    await reportShown();

    const applied = sent(commands, "import_apply");
    expect(applied).toHaveLength(2);
    expect(JSON.stringify(applied[1])).not.toContain(SENTINEL);
    expect(applied[1]).toMatchObject({
      answers: [{ value: null }, { value: null }, { value: null }],
    });
    expect([cacheText(client), pageText()].join()).not.toContain(SENTINEL);
  });

  it("never leave when the wizard closes on step 3 and a new import follows", async () => {
    importBackend();
    const commands = recordCommands();
    const { client, user, onClose, setOpen } = renderWizard();
    await toPasswords(user);
    await user.type(passwordField(), `${SENTINEL}{Enter}`);
    await screen.findByText("2 / 3");
    await user.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
    setOpen(false);

    setOpen(true);
    await toOptions(user);
    await user.click(screen.getByRole("button", { name: "Avanti" }));
    await screen.findByText("1 / 3");
    await user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
    await reportShown();

    expect(carrying(commands)).toEqual([]);
    expect([cacheText(client), pageText()].join()).not.toContain(SENTINEL);
  });
});
