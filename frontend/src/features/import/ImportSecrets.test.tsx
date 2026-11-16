import type { QueryClient } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { importBackend, passwordField, renderWizard, reportShown, toPasswords } from "./testWizard";

/**
 * The passwords typed in the wizard reach the backend only in
 * `import_apply`, and nothing else keeps them (SPEC-UI §1.2), checked with a
 * sentinel secret.
 */

const SENTINEL = "sentinel-import-secret-5e7a";

/** The IPC of the page, which the fake backend installs with `mockIPC`. */
interface TauriInternals {
  invoke: (command: string, args?: unknown, options?: unknown) => Promise<unknown>;
}

/** Records every command the page sends, with its arguments. */
function recordCommands() {
  // The global of Tauri is untyped: this is its shape in `@tauri-apps/api/mocks`.
  const internals = (window as unknown as { __TAURI_INTERNALS__: TauriInternals })
    .__TAURI_INTERNALS__;
  return vi.spyOn(internals, "invoke");
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

    const carrying = commands.mock.calls.filter(([, args]) =>
      JSON.stringify(args ?? null).includes(SENTINEL),
    );
    expect(carrying.map(([command]) => command)).toEqual(["import_apply"]);
    expect(carrying[0]?.[1]).toMatchObject({
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

    const values = [...document.querySelectorAll("input")].map((input) => input.value);
    expect([document.body.innerHTML, ...values].join()).not.toContain(SENTINEL);
  });
});
