import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { texts } from "../../i18n/it";
import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { renderWithQueries } from "../../test/render";
import { PreferencesDialog } from "./PreferencesDialog";

const t = texts.preferences.terminal;

/** The command saved in `config.toml` before the test. */
const SAVED = "kitty -e {cmd}";

/** The preferences with a custom terminal whose command is `SAVED`; `confirmed`, the answer of the native dialog. */
function openWithCommand(confirmed: boolean) {
  const state = initialFakeState();
  state.config["terminal.app"] = "custom";
  state.config["terminal.command"] = SAVED;
  state.dialogs.terminalCommand = confirmed;
  installFakeBackend(state);
  renderWithQueries(<PreferencesDialog open onClose={() => undefined} />);
  return state;
}

/** Replaces the command in the field with `command` and leaves the field. */
async function typeCommand(command: string) {
  const field = await screen.findByLabelText(t.command);
  await userEvent.clear(field);
  await userEvent.type(field, command.replaceAll("{", "{{"));
  await userEvent.tab();
}

describe("TerminalCommandSetting", () => {
  it("saves a new command the user confirms", async () => {
    const state = openWithCommand(true);

    await typeCommand("wezterm start -- {cmd}");

    await waitFor(() => expect(state.config["terminal.command"]).toBe("wezterm start -- {cmd}"));
  });

  it("puts the saved command back, without an error, when the user does not confirm", async () => {
    const state = openWithCommand(false);

    await typeCommand("/opt/example/bin/altro {cmd}");

    await waitFor(() => expect(screen.getByLabelText(t.command)).toHaveValue(SAVED));
    expect(state.config["terminal.command"]).toBe(SAVED);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
