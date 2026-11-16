import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { commands } from "../../bindings";

import { texts } from "../../i18n/it";
import { createQueryClient } from "../../ipc/queryClient";
import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { renderWithQueries } from "../../test/render";
import { PreferencesDialog } from "./PreferencesDialog";
import { useConfigSettings } from "./useConfigSettings";

const t = texts.preferences;

function open(state = initialFakeState()) {
  installFakeBackend(state);
  renderWithQueries(<PreferencesDialog open onClose={() => undefined} />);
  return state;
}

async function choose(group: string, option: string) {
  const radios = await screen.findByRole("radiogroup", { name: group });
  await userEvent.click(within(radios).getByRole("radio", { name: option }));
}

/** The row of the path field `label`, with its buttons. */
async function pathRow(label: string) {
  const field = await screen.findByLabelText(label);
  const row = field.parentElement?.parentElement;
  if (!row) throw new Error(`campo ${label} senza riga`);
  return { field, row: within(row) };
}

describe("PreferencesDialog", () => {
  it("sets a key of the configuration at once", async () => {
    const state = open();

    await choose(t.rdp.cert, t.rdp.certs.deny);

    await waitFor(() => expect(state.config["rdp.cert"]).toBe("deny"));
  });

  it("shows the file of the vault only with the file vault", async () => {
    const state = initialFakeState();
    state.config = { "vault.backend": "keychain" };
    open(state);
    await screen.findByRole("radiogroup", { name: t.vault.label });
    expect(screen.queryByLabelText(t.vault.file)).not.toBeInTheDocument();

    await choose(t.vault.label, t.vault.backends.file);

    expect(await screen.findByLabelText(t.vault.file)).toBeInTheDocument();
    expect(state.config["vault.backend"]).toBe("file");
  });

  it("chooses a program with the native dialog and puts it back to its default", async () => {
    const state = initialFakeState();
    state.dialogs.configFiles["ssh.program"] = "/opt/prova/bin/ssh";
    open(state);
    const { field, row } = await pathRow(t.ssh.label);

    await userEvent.click(row.getByRole("button", { name: t.browse }));
    await waitFor(() => expect(field).toHaveValue("/opt/prova/bin/ssh"));
    await userEvent.click(row.getByRole("button", { name: t.reset }));

    await waitFor(() => expect(field).toHaveValue(""));
    expect(state.config["ssh.program"]).toBeUndefined();
  });

  it("saves the command of a custom terminal when the field is left", async () => {
    const state = initialFakeState();
    state.config["terminal.app"] = "custom";
    open(state);

    await userEvent.type(await screen.findByLabelText(t.terminal.command), "kitty -e {{cmd}");
    await userEvent.tab();

    await waitFor(() => expect(state.config["terminal.command"]).toBe("kitty -e {cmd}"));
  });

  it("sends the command once with Enter and then leaving the field", async () => {
    const state = initialFakeState();
    state.config["terminal.app"] = "custom";
    open(state);
    const configSet = vi.spyOn(commands, "configSet");

    await userEvent.type(
      await screen.findByLabelText(t.terminal.command),
      "kitty -e {{cmd}{Enter}",
    );
    await userEvent.tab();

    await waitFor(() => expect(state.config["terminal.command"]).toBe("kitty -e {cmd}"));
    expect(configSet).toHaveBeenCalledOnce();
    configSet.mockRestore();
  });

  it("saves the appearance in the preferences of the interface", async () => {
    const state = open();

    await choose(t.appearance.density, t.appearance.densities.compact);
    await userEvent.click(screen.getByRole("switch", { name: t.appearance.reduceMotion }));

    await waitFor(() =>
      expect(state.preferences).toMatchObject({ density: "compact", reduceMotion: true }),
    );
  });
});

describe("useConfigSettings", () => {
  it("keeps the message of a refused change for its key", async () => {
    installFakeBackend();
    const client = createQueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useConfigSettings(), { wrapper });

    // A program is a file: the backend refuses it typed.
    act(() => result.current.set("ssh.program", "/usr/bin/ssh"));

    await waitFor(() => expect(result.current.error("ssh.program")).toContain("Scegli"));
    expect(result.current.error("rdp.cert")).toBeUndefined();
  });
});
