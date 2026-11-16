import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useDeleteConfirm } from "../features/details/deleteStore";
import { useSessionsStore } from "../features/sessions/store";
import { texts } from "../i18n/it";
import { createQueryClient } from "../ipc/queryClient";
import type { OpenSessionRequest, SessionTransport } from "../ipc/sessions";
import { useRequests } from "../state/requests";
import { useToasts } from "../state/toasts";
import { useWorkspace } from "../state/workspace";
import { type FakeBackendState, initialFakeState, installFakeBackend } from "../test/fakeBackend";
import { FakeSessionTransport, workingConnection } from "../test/fakeSessions";
import { App } from "./App";
import type { EditorProps, FeatureRegistry, LoginProps } from "./features";

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

function renderApp(
  state: FakeBackendState = initialFakeState(),
  features: FeatureRegistry = {},
  transport: SessionTransport = recordingTransport().transport,
) {
  installFakeBackend(state);
  return render(
    <App queryClient={createQueryClient()} transport={transport} features={features} />,
  );
}

async function tree() {
  return screen.findByRole("tree", { name: texts.tree.label });
}

describe("the window", () => {
  beforeEach(() => {
    // jsdom has no canvas: the background draws nothing.
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
    useWorkspace.getState().documentSwitched();
    useSessionsStore.setState({ sessions: [] });
    useRequests.setState({ active: null });
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

  it("opens the editor of a new folder inside the folder selected", async () => {
    const targets: EditorProps["target"][] = [];
    const Editor = ({ target }: EditorProps) => {
      targets.push(target);
      return <p>editor</p>;
    };
    renderApp(initialFakeState(), { editor: Editor });
    await userEvent.click(within(await tree()).getByRole("treeitem", { name: "Rete" }));

    await userEvent.click(screen.getByRole("button", { name: texts.app.toolbar.newFolder }));

    await screen.findByText("editor");
    expect(targets.at(-1)).toMatchObject({ mode: "add", kind: "folder" });
    expect(targets.at(-1)).toHaveProperty("parent", useWorkspace.getState().selectedId);
  });

  it("asks the login of a credential «ask at launch» and opens the session with it", async () => {
    const Login = ({ onDone }: LoginProps) => (
      <button
        type="button"
        onClick={() => onDone({ username: "operatore", password: "sentinel-login" })}
      >
        accedi
      </button>
    );
    const { transport, opened } = recordingTransport();
    renderApp(initialFakeState(), { login: Login }, transport);
    await userEvent.click(within(await tree()).getByRole("treeitem", { name: "ACME" }));

    await userEvent.dblClick(within(await tree()).getByRole("treeitem", { name: "kiosk" }));
    await userEvent.click(await screen.findByRole("button", { name: "accedi" }));

    await waitFor(() => expect(opened.at(-1)?.login?.username).toBe("operatore"));
    expect(await screen.findByRole("tab", { name: /kiosk.*SSH/ })).toBeInTheDocument();
    expect(JSON.stringify(useRequests.getState())).not.toContain("sentinel-login");
  });

  it("does not connect when the login dialog is not mounted, and says why", async () => {
    const { transport, opened } = recordingTransport();
    renderApp(initialFakeState(), {}, transport);
    await userEvent.click(within(await tree()).getByRole("treeitem", { name: "ACME" }));

    await userEvent.dblClick(within(await tree()).getByRole("treeitem", { name: "kiosk" }));

    expect((await screen.findAllByText(texts.app.connection.noLoginDialog)).length).toBeGreaterThan(
      0,
    );
    expect(opened).toEqual([]);
  });

  it("deletes a node after the confirmation and puts it back with «Annulla»", async () => {
    renderApp();
    await userEvent.click(within(await tree()).getByRole("treeitem", { name: "vnc-legacy" }));
    await userEvent.click(
      await screen.findByRole("button", { name: texts.details.actions.delete }),
    );

    const dialog = await screen.findByRole("dialog");
    await userEvent.click(
      within(dialog).getByRole("button", { name: texts.details.remove.delete }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("treeitem", { name: "vnc-legacy" })).not.toBeInTheDocument(),
    );
    await userEvent.click(screen.getByRole("button", { name: texts.app.toasts.undo }));

    expect(await screen.findByRole("treeitem", { name: "vnc-legacy" })).toBeInTheDocument();
  });

  it("offers «Imposta…» of a secret only with the password dialog mounted", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    renderApp(state);
    await userEvent.click(within(await tree()).getByRole("treeitem", { name: "deploy" }));

    const secrets = await screen.findByRole("region", { name: texts.details.secrets.heading });

    for (const button of within(secrets).getAllByRole("button", { name: /Imposta|Sostituisci/ })) {
      expect(button).toHaveAttribute("aria-disabled", "true");
    }
  });
});
