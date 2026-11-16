import type { QueryClient } from "@tanstack/react-query";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { commands } from "../bindings";
import { useDeleteConfirm } from "../features/details/deleteStore";
import { useSessionsStore } from "../features/sessions/store";
import { texts } from "../i18n/it";
import { createQueryClient } from "../ipc/queryClient";
import type { OpenSessionRequest, SessionTransport } from "../ipc/sessions";
import { requestVault, useRequests } from "../state/requests";
import { useToasts } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import {
  type FakeBackendState,
  initialFakeState,
  installFakeBackend,
  TEST_MASTER_PASSWORD,
} from "../test/fakeBackend";
import { FakeSessionTransport, workingConnection } from "../test/fakeSessions";
import { App } from "./App";
import type {
  EditorProps,
  FeatureRegistry,
  LoginProps,
  PasswordProps,
  VaultDialogProps,
} from "./features";

// xterm.js needs a real layout engine: the e2e runs the real terminal in WebKit.
vi.mock("../components/terminal/Terminal", () => ({
  Terminal: ({ label }: { label: string }) => <div role="group" aria-label={label} />,
}));

/** A transport that works for every node and records what opening received. */
function recordingTransport(): { transport: SessionTransport; opened: OpenSessionRequest[] } {
  const fake = new FakeSessionTransport({}, workingConnection);
  const opened: OpenSessionRequest[] = [];
  return {
    opened,
    transport: {
      open: (request, handlers) => {
        opened.push(request);
        return fake.open(request, handlers);
      },
      input: (id, data) => fake.input(id, data),
      resize: (id, size) => fake.resize(id, size),
      close: (id) => fake.close(id),
    },
  };
}

let queryClient: QueryClient;

function renderApp(
  state: FakeBackendState = initialFakeState(),
  features: FeatureRegistry = {},
  transport: SessionTransport = recordingTransport().transport,
) {
  installFakeBackend(state);
  queryClient = createQueryClient();
  return render(<App queryClient={queryClient} transport={transport} features={features} />);
}

async function tree() {
  return screen.findByRole("tree", { name: texts.tree.label });
}

/** The row of node `name`: its accessible name starts with the name. */
async function row(name: string) {
  return within(await tree()).findByRole("treeitem", { name: new RegExp(`^${name},`) });
}

/** A login dialog that answers with the sentinel password. */
const Login = ({ onDone }: LoginProps) => (
  <button type="button" onClick={() => onDone({ username: "operatore", password: SENTINEL })}>
    accedi
  </button>
);

/** An unlock dialog that opens the fake vault with the test master password. */
const Unlock = ({ onDone }: VaultDialogProps) => (
  <button
    type="button"
    onClick={() => void commands.unlockVault(TEST_MASTER_PASSWORD).then(() => onDone(true))}
  >
    sblocca
  </button>
);

const SENTINEL = "sentinel-login";

describe("the window", () => {
  beforeEach(() => {
    // jsdom has no canvas: the background draws nothing.
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
    useWorkspace.getState().documentSwitched();
    useSessionsStore.setState({ sessions: [] });
    useRequests.setState({ stack: [] });
    useToasts.setState({ toasts: [] });
    useDeleteConfirm.getState().dismiss();
  });

  afterEach(async () => {
    // The listeners of the events let go once their subscription resolves:
    // before the fake IPC goes away with the mocks.
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
  });

  it("starts on the first run without a document", async () => {
    const state = initialFakeState();
    state.document = null;
    state.recent = [];

    renderApp(state);

    expect(await screen.findByText(texts.firstRun.lead)).toBeInTheDocument();
  });

  it("creates a new document from the first run and shows it empty", async () => {
    const state = initialFakeState();
    state.document = null;
    state.recent = [];
    state.dialogs.newDocument = "/Users/utente-di-prova/infra/nuovo.json";
    renderApp(state);

    await userEvent.click(
      await screen.findByRole("button", { name: new RegExp(texts.firstRun.create.title) }),
    );

    expect(
      await screen.findByRole("heading", { name: texts.firstRun.empty.title }),
    ).toBeInTheDocument();
    expect(state.files).toHaveProperty(["/Users/utente-di-prova/infra/nuovo.json"]);
  });

  it("opens the document opened last at startup", async () => {
    const state = initialFakeState();
    state.document = null;

    renderApp(state);

    expect(await tree()).toBeInTheDocument();
  });

  it("opens the default document when the last one is gone", async () => {
    const state = initialFakeState();
    const sample = state.document;
    state.document = null;
    state.recent = ["/Users/utente-di-prova/infra/spostato.json"];
    if (sample) state.files[state.defaultDocumentPath] = sample;

    renderApp(state);

    expect(await tree()).toBeInTheDocument();
  });

  it("disables a function that is not mounted, with its reason", async () => {
    renderApp();
    await tree();

    const button = screen.getByRole("button", { name: texts.app.toolbar.import });

    expect(button).toHaveAttribute("aria-disabled", "true");
  });

  it("mounts the function of a hook point when asked, and closes it", async () => {
    const Import = vi.fn(({ onClose }: { onClose: () => void }) => (
      <button type="button" onClick={onClose}>
        chiudi import
      </button>
    ));
    renderApp(initialFakeState(), { import: Import });
    await tree();

    await userEvent.click(screen.getByRole("button", { name: texts.app.toolbar.import }));
    await userEvent.click(await screen.findByRole("button", { name: "chiudi import" }));

    expect(screen.queryByRole("button", { name: "chiudi import" })).not.toBeInTheDocument();
  });

  it("keeps the editor mounted under the unlock it asks for", async () => {
    const Editor = () => (
      <button type="button" onClick={() => void requestVault()}>
        salva
      </button>
    );
    renderApp(initialFakeState(), { editor: Editor, unlockVault: Unlock });
    await userEvent.click(await row("Rete"));
    await userEvent.click(screen.getByRole("button", { name: texts.app.toolbar.newFolder }));

    await userEvent.click(await screen.findByRole("button", { name: "salva" }));

    expect(await screen.findByRole("button", { name: "sblocca" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "salva" })).toBeInTheDocument();
  });

  it("opens the editor of a new folder inside the folder selected", async () => {
    const targets: EditorProps["target"][] = [];
    const Editor = ({ target }: EditorProps) => {
      targets.push(target);
      return <p>editor</p>;
    };
    renderApp(initialFakeState(), { editor: Editor });
    await userEvent.click(await row("Rete"));

    await userEvent.click(screen.getByRole("button", { name: texts.app.toolbar.newFolder }));

    await screen.findByText("editor");
    expect(targets.at(-1)).toMatchObject({ mode: "add", kind: "folder" });
    expect(targets.at(-1)).toHaveProperty("parent", useWorkspace.getState().selectedId);
  });

  it("unlocks the vault before «Imposta…» of a secret, then asks the password", async () => {
    const Password = ({ field }: PasswordProps) => <p>password {field.kind}</p>;
    renderApp(initialFakeState(), { password: Password, unlockVault: Unlock });
    await userEvent.click(await row("Admin ACME"));
    const secrets = await screen.findByRole("region", { name: texts.details.secrets.heading });

    await userEvent.click(
      within(secrets).getAllByRole("button", { name: /Imposta|Sostituisci/ })[0]!,
    );
    await userEvent.click(await screen.findByRole("button", { name: "sblocca" }));

    expect(await screen.findByText("password password")).toBeInTheDocument();
  });

  it("repeats a command refused with the vault locked once it is unlocked", async () => {
    renderApp(initialFakeState(), { unlockVault: Unlock });
    await userEvent.click(await row("Admin ACME"));

    await userEvent.click(
      await screen.findByRole("button", { name: texts.details.actions.duplicate }),
    );
    await userEvent.click(await screen.findByRole("button", { name: "sblocca" }));

    expect(await row("Admin ACME \\(copia\\)")).toBeInTheDocument();
  });

  it("creates the file vault, not unlocks it, when its file does not exist yet", async () => {
    const Create = ({ onDone }: VaultDialogProps) => (
      <button
        type="button"
        onClick={() => void commands.createFileVault(TEST_MASTER_PASSWORD).then(() => onDone(true))}
      >
        crea vault
      </button>
    );
    const Password = ({ field }: PasswordProps) => <p>password {field.kind}</p>;
    const state = initialFakeState();
    state.vault.exists = false;
    renderApp(state, { password: Password, unlockVault: Unlock, createFileVault: Create });
    await userEvent.click(await row("Admin ACME"));
    const secrets = await screen.findByRole("region", { name: texts.details.secrets.heading });

    await userEvent.click(
      within(secrets).getAllByRole("button", { name: /Imposta|Sostituisci/ })[0]!,
    );
    await userEvent.click(await screen.findByRole("button", { name: "crea vault" }));

    expect(await screen.findByText("password password")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "sblocca" })).not.toBeInTheDocument();
  });

  it("answers «no» at once to a request of the code whose function is not mounted", async () => {
    const answers: boolean[] = [];
    const Editor = () => (
      <button type="button" onClick={() => void requestVault().then((done) => answers.push(done))}>
        salva
      </button>
    );
    renderApp(initialFakeState(), { editor: Editor });
    await userEvent.click(await row("Rete"));
    await userEvent.click(screen.getByRole("button", { name: texts.app.toolbar.newFolder }));

    await userEvent.click(await screen.findByRole("button", { name: "salva" }));

    await waitFor(() => expect(answers).toEqual([false]));
    expect(useRequests.getState().stack.map((open) => open.request.kind)).toEqual(["editor"]);
  });

  describe("with a credential «ask at launch»", () => {
    async function connectToKiosk(transport: SessionTransport) {
      renderApp(initialFakeState(), { login: Login }, transport);
      await userEvent.click(await row("ACME"));
      await userEvent.dblClick(await row("kiosk"));
    }

    it("opens the session with the login asked", async () => {
      const { transport, opened } = recordingTransport();
      await connectToKiosk(transport);

      await userEvent.click(await screen.findByRole("button", { name: "accedi" }));

      await waitFor(() => expect(opened.at(-1)?.login?.username).toBe("operatore"));
      expect(await screen.findByRole("tab", { name: /kiosk.*SSH/ })).toBeInTheDocument();
    });

    it("keeps the password out of every store while the dialog is open and after", async () => {
      const seen: string[] = [];
      const record = (state: unknown) => seen.push(JSON.stringify(state));
      const unsubscribe = [
        useRequests.subscribe(record),
        useWorkspace.subscribe(record),
        useSessionsStore.subscribe(record),
        useToasts.subscribe(record),
      ];
      await connectToKiosk(recordingTransport().transport);

      await userEvent.click(await screen.findByRole("button", { name: "accedi" }));
      await screen.findByRole("tab", { name: /kiosk.*SSH/ });

      unsubscribe.forEach((stop) => stop());
      expect(seen.join("\n")).not.toContain(SENTINEL);
    });

    it("keeps the password out of the cache of the queries", async () => {
      await connectToKiosk(recordingTransport().transport);

      await userEvent.click(await screen.findByRole("button", { name: "accedi" }));
      await screen.findByRole("tab", { name: /kiosk.*SSH/ });

      const cached = queryClient
        .getQueryCache()
        .getAll()
        .map((query) => JSON.stringify(query.state.data));
      const mutations = queryClient
        .getMutationCache()
        .getAll()
        .map((mutation) => JSON.stringify(mutation.state.variables));
      expect([...cached, ...mutations].join("\n")).not.toContain(SENTINEL);
    });
  });

  it("does not connect when the login dialog is not mounted, and says why", async () => {
    const { transport, opened } = recordingTransport();
    renderApp(initialFakeState(), {}, transport);
    await userEvent.click(await row("ACME"));

    await userEvent.dblClick(await row("kiosk"));

    expect((await screen.findAllByText(texts.app.connection.noLoginDialog)).length).toBeGreaterThan(
      0,
    );
    expect(opened).toEqual([]);
  });

  it("deletes a node after the confirmation and puts it back with «Annulla»", async () => {
    renderApp();
    await userEvent.click(await row("vnc-legacy"));
    await userEvent.click(
      await screen.findByRole("button", { name: texts.details.actions.delete }),
    );

    const dialog = await screen.findByRole("dialog");
    await userEvent.click(
      within(dialog).getByRole("button", { name: texts.details.remove.delete }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("treeitem", { name: /^vnc-legacy,/ })).not.toBeInTheDocument(),
    );
    await userEvent.click(screen.getByRole("button", { name: texts.app.toasts.undo }));

    expect(await row("vnc-legacy")).toBeInTheDocument();
  });

  it("offers «Imposta…» of a secret only with the password dialog mounted", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    renderApp(state);
    await userEvent.click(await row("deploy"));

    const secrets = await screen.findByRole("region", { name: texts.details.secrets.heading });

    for (const button of within(secrets).getAllByRole("button", { name: /Imposta|Sostituisci/ })) {
      expect(button).toHaveAttribute("aria-disabled", "true");
    }
  });

  it("names a row with what it shows, for the screen readers", async () => {
    renderApp();

    expect(await row("bastion")).toHaveAccessibleName(
      "bastion, Connessione SSH, bastion.example.com",
    );
  });
});
