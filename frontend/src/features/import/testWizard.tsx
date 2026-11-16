import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, vi } from "vitest";

import { FeatureHost } from "../../app/FeatureHost";
import { type FeatureRegistry, FeatureRegistryContext, type ImportProps } from "../../app/features";
import { createQueryClient } from "../../ipc/queryClient";
import { closeRequest, openRequest, topRequest, useRequests } from "../../state/requests";
import {
  type FakeBackendState,
  initialFakeState,
  installFakeBackend,
} from "../../test/fakeBackend";
import { IMPORT_FILE } from "../../test/fakeBackendImport";
import { ImportWizard } from "./ImportWizard";
import { TestUnlockVault } from "./TestUnlockVault";

/** Helpers of the tests of the wizard, on the fake backend with the fake Royal document. */

/** The fake backend with the Royal document as the answer of the import dialog. */
export function importBackend(
  options: { file?: string | null; vaultLocked?: boolean } = {},
): FakeBackendState {
  const state = initialFakeState();
  state.vault.locked = options.vaultLocked ?? false;
  state.dialogs.importFile = options.file === undefined ? IMPORT_FILE : options.file;
  state.dialogs.report = "/Users/utente-di-prova/Documenti/report-import.json";
  return installFakeBackend(state);
}

/**
 * The wizard opened as the window opens it: a request of the stack, mounted
 * by `FeatureHost` with the requests it makes on top (the unlock of the vault
 * is the stand-in of the dialog of UI-WP5). Spies for its callbacks, the
 * query client it uses and `setOpen` to close it and open it again.
 */
export function renderWizard(features: FeatureRegistry = { unlockVault: TestUnlockVault }) {
  const client: QueryClient = createQueryClient();
  const onClose = vi.fn();
  const onGoToNode = vi.fn();
  const user = userEvent.setup();
  // The wizard at the hook point, with spies in front of the callbacks of the window.
  const SpiedImport = (props: ImportProps) => (
    <ImportWizard
      onClose={() => {
        onClose();
        props.onClose();
      }}
      onGoToNode={(id) => {
        onGoToNode(id);
        props.onGoToNode(id);
      }}
    />
  );
  // The requests of the window are a store of the page: each test starts without any.
  useRequests.setState({ stack: [] });
  render(
    <QueryClientProvider client={client}>
      <FeatureRegistryContext value={{ ...features, import: SpiedImport }}>
        <FeatureHost />
      </FeatureRegistryContext>
    </QueryClientProvider>,
  );
  const setOpen = (open: boolean) =>
    act(() => {
      const top = topRequest();
      if (open) openRequest({ kind: "import" });
      else if (top) closeRequest(top.id);
    });
  setOpen(true);
  return { client, onClose, onGoToNode, user, setOpen };
}

/** The IPC of the page, which the fake backend installs with `mockIPC`. */
interface TauriInternals {
  invoke: (command: string, args?: unknown, options?: unknown) => Promise<unknown>;
}

function tauriInternals(): TauriInternals {
  // The global of Tauri is untyped: this is its shape in `@tauri-apps/api/mocks`.
  return (window as unknown as { __TAURI_INTERNALS__: TauriInternals }).__TAURI_INTERNALS__;
}

/** Records every command the page sends, with its arguments. */
export function recordCommands() {
  return vi.spyOn(tauriInternals(), "invoke");
}

/** Runs `command` through `around`, which gets the answer of the fake backend to change or delay. */
export function interceptCommand(
  command: string,
  around: (answer: () => Promise<unknown>) => Promise<unknown>,
) {
  const internals = tauriInternals();
  const original = internals.invoke.bind(internals);
  return vi
    .spyOn(internals, "invoke")
    .mockImplementation((name, args, options) =>
      name === command
        ? around(() => original(name, args, options))
        : original(name, args, options),
    );
}

/** The commands named `command` that were sent, with their arguments. */
export function sent(commands: ReturnType<typeof recordCommands>, command: string): unknown[] {
  return commands.mock.calls.filter(([name]) => name === command).map(([, args]) => args);
}

type User = ReturnType<typeof userEvent.setup>;

/** Chooses the file and waits for it to be read. */
export async function chooseFile(user: User): Promise<void> {
  await user.click(screen.getByRole("button", { name: "Scegli file…" }));
  await screen.findByText("Leggibile");
}

/** From the start to step 2, with its dry run done. */
export async function toOptions(user: User): Promise<void> {
  await chooseFile(user);
  await user.click(screen.getByRole("button", { name: "Avanti" }));
  await screen.findByText(/^In prova:/);
}

/** From the start to the first password of step 3. */
export async function toPasswords(user: User): Promise<void> {
  await toOptions(user);
  await user.click(screen.getByRole("button", { name: "Avanti" }));
  await screen.findByText("1 / 3");
}

/** The password field of step 3. */
export function passwordField(): HTMLInputElement {
  const field = screen.getByLabelText("Password");
  if (!(field instanceof HTMLInputElement)) throw new Error("il campo password non è un input");
  return field;
}

/** Waits for the report of step 4. */
export async function reportShown(): Promise<void> {
  await waitFor(() => expect(screen.getByText(/^Importazione completata/)).toBeInTheDocument());
}
