import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, vi } from "vitest";

import { FeatureHost } from "../../app/FeatureHost";
import { FeatureRegistryContext, type ImportProps } from "../../app/features";
import type { ImportAnswerInput_Deserialize, ImportPreviewView_Serialize } from "../../bindings";
import { createQueryClient } from "../../ipc/queryClient";
import { closeRequest, openRequest, topRequest, useRequests } from "../../state/requests";
import {
  type FakeBackendState,
  initialFakeState,
  installFakeBackend,
  TEST_MASTER_PASSWORD,
} from "../../test/fakeBackend";
import { IMPORT_FILE } from "../../test/fakeBackendImport";
import { UnlockVaultFeature } from "../dialogs/DialogFeatures";
import { ImportWizard } from "./ImportWizard";

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
 * by `FeatureHost` with the requests it makes on top (the unlock dialog of
 * the vault, as the window registers it). Spies for its callbacks, the query
 * client it uses and `setOpen` to close it and open it again.
 */
export function renderWizard() {
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
      <FeatureRegistryContext value={{ unlockVault: UnlockVaultFeature, import: SpiedImport }}>
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

/**
 * Makes the first request of the fake Royal document ask for a private key,
 * as a Royal credential with its key in the document does: the fake asks
 * passwords only, so the request changes on its way to the page and its
 * answer on its way back. Returns the answers the page sent to `import_apply`.
 */
export function askPrivateKeyFirst(): ImportAnswerInput_Deserialize[][] {
  const internals = tauriInternals();
  const original = internals.invoke.bind(internals);
  const applied: ImportAnswerInput_Deserialize[][] = [];
  const first = <T extends { field: unknown }>(entries: T[], field: T["field"]) =>
    entries.map((entry, index) => (index === 0 ? { ...entry, field } : entry));
  vi.spyOn(internals, "invoke").mockImplementation(async (name, args, options) => {
    if (name === "import_preview") {
      // The answer of the fake to this command, which the bindings type.
      const preview = (await original(name, args, options)) as ImportPreviewView_Serialize;
      return { ...preview, requests: first(preview.requests, { kind: "privateKey" }) };
    }
    if (name === "import_apply") {
      // The arguments the bindings send to this command.
      const { answers, ...rest } = args as { answers: ImportAnswerInput_Deserialize[] };
      applied.push(answers);
      return original(name, { ...rest, answers: first(answers, { kind: "password" }) }, options);
    }
    return original(name, args, options);
  });
  return applied;
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

/** The private key area of step 3. */
export function keyField(): HTMLTextAreaElement {
  const field = screen.getByLabelText("Chiave privata");
  if (!(field instanceof HTMLTextAreaElement)) throw new Error("la chiave non è una textarea");
  return field;
}

/** Opens the locked vault with the unlock dialog, which the wizard asks for. */
export async function unlockVault(user: User): Promise<void> {
  const dialog = await screen.findByRole("dialog", { name: "Sblocca il vault" });
  await user.type(within(dialog).getByLabelText("Master password"), TEST_MASTER_PASSWORD);
  await user.click(within(dialog).getByRole("button", { name: "Sblocca" }));
}

/** Waits for the report of step 4. */
export async function reportShown(): Promise<void> {
  await waitFor(() => expect(screen.getByText(/^Importazione completata/)).toBeInTheDocument());
}
