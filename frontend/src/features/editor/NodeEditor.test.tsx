import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, renderHook, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { emit } from "@tauri-apps/api/event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { useReloadDocument } from "../../ipc/mutations";
import { useRequests } from "../../state/requests";
import { useToasts } from "../../state/toasts";
import { useWorkspace } from "../../state/workspace";
import {
  type FakeBackendState,
  TEST_MASTER_PASSWORD,
  initialFakeState,
  installFakeBackend,
} from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { type FakeNode, blankNode, walk } from "../../test/fakeBackendModel";
import { renderInWindow } from "../../test/renderInWindow";
import { useDeleteConfirm } from "../details/deleteStore";
import { useSessionsStore } from "../sessions/store";
import { type EditorRequest, NodeEditor } from "./NodeEditor";

const t = texts.editor;
const secret = texts.dialogs.secret;
/** A sentinel secret: it must never be found in the page or in the cache of Query. */
const SENTINEL = "sentinel-editor-secret-3fa9";

function nodeNamed(state: FakeBackendState, name: string): FakeNode {
  const found = [...walk(state.document?.nodes ?? [])].find(({ node }) => node.name === name);
  if (!found) throw new Error(`nodo ${name} assente`);
  return found.node;
}

/** The editor as the window mounts it, with the dialogs of the vault on top of it. */
function renderEditor(request: EditorRequest) {
  const onSaved = vi.fn();
  const onClose = vi.fn();
  const editor = (shown: EditorRequest | null) => (
    <NodeEditor request={shown} onClose={onClose} onSaved={onSaved} />
  );
  const { client, rerender } = renderInWindow(editor(request));
  /** Closes the editor, as the host does: it is unmounted. */
  const unmountEditor = () => rerender(editor(null));
  return { client, onSaved, onClose, unmountEditor };
}

/** The sentinel is nowhere: page, values of the fields, cache of Query, stores of the window. */
function expectNoSentinel(client: QueryClient) {
  expect(document.body.innerHTML).not.toContain(SENTINEL);
  const fields = [
    ...document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>("input, textarea"),
  ];
  expect(fields.map((field) => field.value)).not.toContain(SENTINEL);
  expect(cacheText(client)).not.toContain(SENTINEL);
  expect(storesText()).not.toContain(SENTINEL);
}

/** Everything the stores of the window hold, as text. */
function storesText(): string {
  return JSON.stringify([
    useRequests.getState(),
    useToasts.getState(),
    useWorkspace.getState(),
    useSessionsStore.getState(),
    useDeleteConfirm.getState(),
  ]);
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

async function type(label: string, text: string) {
  const field = await screen.findByLabelText(label);
  await userEvent.clear(field);
  if (text !== "") await userEvent.type(field, text);
}

/** The sample document with a second "dc01" in "Rete", and the id of the first one. */
function withCopyInRete(): { state: FakeBackendState; id: string } {
  const state = initialFakeState();
  state.vault.locked = false;
  const { id } = nodeNamed(state, "dc01");
  nodeNamed(state, "Rete").children.push(blankNode(state, "connection", "dc01"));
  installFakeBackend(state);
  return { state, id };
}

/** Chooses `name` as the folder of the node with "Cambia…". */
async function chooseFolder(name: string) {
  await userEvent.click(await screen.findByRole("button", { name: t.general.change }));
  const picker = await screen.findByRole("dialog", { name: texts.dialogs.picker.folder.title });
  await userEvent.click(within(picker).getByRole("option", { name: new RegExp(name) }));
  await userEvent.click(
    within(picker).getByRole("button", { name: new RegExp(`^Scegli ${name}`) }),
  );
}

/** Types the master password in the unlock dialog that "Salva" opens. */
async function unlockVault() {
  await userEvent.type(
    await screen.findByLabelText(texts.dialogs.unlock.field),
    TEST_MASTER_PASSWORD,
  );
  await userEvent.click(screen.getByRole("button", { name: texts.dialogs.unlock.unlock }));
}

/** Another program changes the document: the editor shows its notice. */
async function announceChangeFromOutside() {
  await act(() => emit("document-modified-externally", { fileName: "portolano.json" }));
  expect(await screen.findByText(t.external.modified)).toBeInTheDocument();
}

/** "Ricarica" of the window, on the cache of the editor. */
async function reloadDocument(client: QueryClient) {
  const { result } = renderHook(() => useReloadDocument(), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    ),
  });
  await act(() => result.current.mutateAsync());
}

async function save() {
  await userEvent.click(screen.getByRole("button", { name: t.save }));
}

/** Stages `value` as the password of the credential being edited. */
async function stagePassword(value: string) {
  await userEvent.click(screen.getByRole("button", { name: t.next }));
  const row = screen.getByRole("group", { name: t.credential.password });
  await userEvent.click(within(row).getByRole("button", { name: /^(Imposta|Sostituisci)…/ }));
  const dialog = screen.getByRole("dialog", { name: /password/ });
  await userEvent.type(within(dialog).getByLabelText(secret.fields.password), value);
  await userEvent.type(within(dialog).getByLabelText(secret.confirm), value);
  await userEvent.click(screen.getByRole("button", { name: secret.stage }));
}

describe("NodeEditor", () => {
  it("creates an SSH connection without a port nor example values", async () => {
    const state = unlocked();
    const parent = await idOf("ACME");
    const { onSaved } = renderEditor({ mode: "add", kind: "ssh", parent });

    await type(t.general.name, "web09");
    await userEvent.click(screen.getByRole("button", { name: t.next }));
    expect(screen.queryByRole("list", { name: t.ssh.jumps })).toBeEmptyDOMElement();
    await type(t.connection.host, "web09.example.com");
    await save();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    const created = nodeNamed(state, "web09");
    expect(created).toMatchObject({ host: "web09.example.com", port: null, settings: {} });
  });

  it("saves from the first step, with Salva off until something changes", async () => {
    const state = unlocked();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("web01") });
    const button = await screen.findByRole("button", { name: t.save });
    expect(button).toHaveAttribute("aria-disabled", "true");

    await type(t.general.name, "web01-nuovo");
    await save();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "web01-nuovo").host).toBe("web01.example.com");
  });

  it("names the panel as the edit of a node while it loads, then by the node", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });

    expect(screen.getByRole("dialog", { name: t.label.loading })).toBeInTheDocument();
    expect(await screen.findByRole("dialog", { name: "Modifica web01" })).toBeInTheDocument();
  });

  it("puts the focus on Nome once the node has loaded", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });

    const name = await screen.findByLabelText(t.general.name);
    await waitFor(() => expect(name).toHaveFocus());
  });

  it("puts the focus on the first field of the step shown", async () => {
    unlocked();
    renderEditor({ mode: "add", kind: "ssh", parent: null });

    await userEvent.click(screen.getByRole("button", { name: t.next }));
    expect(screen.getByLabelText(t.connection.host)).toHaveFocus();
    await userEvent.click(screen.getByRole("button", { name: t.back }));
    expect(screen.getByLabelText(t.general.name)).toHaveFocus();
  });

  it("puts the focus on the field of an error of the backend", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("dc01") });

    await type(t.general.name, "web01");
    await save();

    const name = await screen.findByLabelText(t.general.name);
    await waitFor(() => expect(name).toHaveAccessibleDescription(/esiste già/));
    expect(name).toHaveFocus();
  });

  it("shows the step of an error of the backend with the focus on its field", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    await userEvent.click(screen.getByRole("button", { name: t.ssh.addOption }));
    const table = screen.getByRole("table", { name: t.ssh.options });
    await userEvent.type(
      within(table).getByLabelText(texts.components.keyValue.name),
      "ProxyCommand",
    );
    await userEvent.click(screen.getByRole("button", { name: t.back }));

    await save();

    const refused = await screen.findByDisplayValue("ProxyCommand");
    await waitFor(() => expect(refused).toHaveFocus());
  });

  it("asks for the host before saving a new connection", async () => {
    unlocked();
    renderEditor({ mode: "add", kind: "rdp", parent: null });

    await type(t.general.name, "dc09");
    await save();

    expect(await screen.findByLabelText(t.connection.host)).toHaveAccessibleDescription(
      t.errors.host,
    );
  });

  it("shows a name already taken under the name", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("dc01") });

    await type(t.general.name, "web01");
    await save();

    expect(await screen.findByLabelText(t.general.name)).toHaveAccessibleDescription(
      /esiste già un nodo chiamato 'web01'/,
    );
  });

  it("sends a staged password only on Salva, after unlocking the vault", async () => {
    const state = installFakeBackend();
    const { client, onSaved } = renderEditor({ mode: "edit", id: await idOf("deploy") });
    await screen.findByLabelText(t.general.name);

    await stagePassword(SENTINEL);
    const row = screen.getByRole("group", { name: t.credential.password });
    expect(within(row).getByText(texts.components.secretBadge.staged)).toBeInTheDocument();
    expect(nodeNamed(state, "deploy").secrets["password"]).toBeUndefined();
    await save();
    await unlockVault();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "deploy").secrets["password"]).toBe(true);
    expectNoSentinel(client);
  });

  it("drops the staged secrets when Salva fails, and says so", async () => {
    const state = unlocked();
    const { client } = renderEditor({ mode: "edit", id: await idOf("deploy") });
    await screen.findByLabelText(t.general.name);
    await stagePassword(SENTINEL);
    await userEvent.click(screen.getByRole("button", { name: t.back }));

    // "bastion" is a sibling of "deploy": the backend refuses the name.
    await type(t.general.name, "bastion");
    await save();

    expect(await screen.findByText(t.errors.secretsLost)).toBeInTheDocument();
    expectNoSentinel(client);
    await type(t.general.name, "deploy-2");
    await save();
    await waitFor(() => expect(nodeNamed(state, "deploy-2")).toBeDefined());
    expect(nodeNamed(state, "deploy-2").secrets["password"]).toBeUndefined();
  });

  it("drops the staged secrets with Annulla, and keeps none once unmounted", async () => {
    const state = unlocked();
    const { client, onClose, unmountEditor } = renderEditor({
      mode: "edit",
      id: await idOf("deploy"),
    });
    await screen.findByLabelText(t.general.name);
    await stagePassword(SENTINEL);

    await userEvent.click(screen.getByRole("button", { name: t.cancel }));
    expect(onClose).toHaveBeenCalledOnce();
    expectNoSentinel(client);
    unmountEditor();

    expectNoSentinel(client);
    expect(nodeNamed(state, "deploy").secrets["password"]).toBeUndefined();
  });

  it("sends nothing when the vault is not unlocked at Salva", async () => {
    const state = installFakeBackend();
    const { client, onSaved } = renderEditor({ mode: "edit", id: await idOf("deploy") });
    await screen.findByLabelText(t.general.name);
    await stagePassword(SENTINEL);

    await save();
    await userEvent.click(
      await screen.findByRole("button", { name: texts.dialogs.unlock.withoutSecrets }),
    );

    expect(onSaved).not.toHaveBeenCalled();
    expect(nodeNamed(state, "deploy").secrets["password"]).toBeUndefined();
    // Still staged: nothing left the editor (DEC-040.4).
    const row = screen.getByRole("group", { name: t.credential.password });
    expect(within(row).getByText(texts.components.secretBadge.staged)).toBeInTheDocument();
    expectNoSentinel(client);
  });

  it("refuses a new secret property without its value", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });

    await userEvent.click(await screen.findByRole("button", { name: t.general.addSecretProperty }));
    const table = screen.getByRole("table", { name: t.general.properties });
    await userEvent.type(within(table).getByLabelText(texts.components.keyValue.name), "Token API");
    await save();

    expect(await screen.findByText(t.errors.secretValue)).toBeInTheDocument();
  });

  it("applies Cancella only on Salva", async () => {
    const state = unlocked();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("Admin ACME") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    const row = screen.getByRole("group", { name: t.credential.password });

    await userEvent.click(
      within(row).getByRole("button", { name: texts.components.secretRow.clear }),
    );
    expect(nodeNamed(state, "Admin ACME").secrets["password"]).toBe(true);
    await save();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "Admin ACME").secrets["password"]).toBeUndefined();
  });

  it("shows the secrets of a locked vault as Vault bloccato", async () => {
    installFakeBackend();
    renderEditor({ mode: "edit", id: await idOf("Admin ACME") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));

    const row = screen.getByRole("group", { name: t.credential.password });
    expect(within(row).getByText(texts.components.secretBadge.locked)).toBeInTheDocument();
  });

  it("replaces a saved password with the vault locked, and unlocks it on Salva", async () => {
    const state = installFakeBackend();
    const { client, onSaved } = renderEditor({ mode: "edit", id: await idOf("Admin ACME") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    const row = screen.getByRole("group", { name: t.credential.password });

    await userEvent.click(
      within(row).getByRole("button", { name: texts.components.secretRow.replace }),
    );
    const dialog = screen.getByRole("dialog", { name: secret.replaceTitles.password });
    await userEvent.type(within(dialog).getByLabelText(secret.fields.password), SENTINEL);
    await userEvent.type(within(dialog).getByLabelText(secret.confirm), SENTINEL);
    await userEvent.click(within(dialog).getByRole("button", { name: secret.stage }));
    expect(within(row).getByText(texts.components.secretBadge.staged)).toBeInTheDocument();
    await save();
    await unlockVault();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(state.vault.locked).toBe(false);
    expect(nodeNamed(state, "Admin ACME").secrets["password"]).toBe(true);
    expectNoSentinel(client);
  });

  it("clears a saved password with the vault locked, and unlocks it on Salva", async () => {
    const state = installFakeBackend();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("Admin ACME") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    const row = screen.getByRole("group", { name: t.credential.password });

    await userEvent.click(
      within(row).getByRole("button", { name: texts.components.secretRow.clear }),
    );
    await save();
    await unlockVault();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "Admin ACME").secrets["password"]).toBeUndefined();
  });

  it("does not save the editor with Fatto in the credential selector", async () => {
    unlocked();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.name, "web01-nuovo");
    await userEvent.click(screen.getByRole("button", { name: t.next }));

    await userEvent.click(
      screen.getByRole("button", { name: texts.components.credentialField.change }),
    );
    await userEvent.click(
      await screen.findByRole("button", { name: texts.components.credentialSelector.done }),
    );

    expect(
      screen.queryByRole("radiogroup", { name: texts.components.credentialSelector.modes }),
    ).not.toBeInTheDocument();
    expect(onSaved).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: /web01/ })).toBeInTheDocument();
  });

  it("saves once when a second submit comes while saving", async () => {
    unlocked();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.name, "web01-nuovo");

    const form = screen.getByLabelText(t.general.name).closest("form");
    if (!form) throw new Error("editor senza form");

    // The second arrives while the first save runs.
    fireEvent.submit(form);
    fireEvent.submit(form);

    await waitFor(() => expect(onSaved).toHaveBeenCalled());
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(onSaved).toHaveBeenCalledOnce();
  });

  it("keeps a new secret property apart from one added after going to the next step and back", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });
    const table = await screen.findByRole("table", { name: t.general.properties });
    await userEvent.click(screen.getByRole("button", { name: t.general.addSecretProperty }));
    await userEvent.type(within(table).getByLabelText(texts.components.keyValue.name), "Token A");
    await userEvent.click(screen.getByRole("button", { name: t.next }));
    await userEvent.click(screen.getByRole("button", { name: t.back }));

    await userEvent.click(screen.getByRole("button", { name: t.general.addSecretProperty }));
    const [first, second] = within(
      screen.getByRole("table", { name: t.general.properties }),
    ).getAllByLabelText(texts.components.keyValue.name);
    if (!first || !second) throw new Error("righe mancanti");
    await userEvent.type(second, "Token B");

    expect(first).toHaveValue("Token A");
    expect(second).toHaveValue("Token B");
  });

  it("keeps a new ssh option apart from one added after going back and forth", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    await userEvent.click(screen.getByRole("button", { name: t.ssh.addOption }));
    await userEvent.type(
      within(screen.getByRole("table", { name: t.ssh.options })).getByLabelText(
        texts.components.keyValue.name,
      ),
      "ServerAliveCountMax",
    );
    await userEvent.click(screen.getByRole("button", { name: t.back }));
    await userEvent.click(screen.getByRole("button", { name: t.next }));

    await userEvent.click(screen.getByRole("button", { name: t.ssh.addOption }));
    const keys = within(screen.getByRole("table", { name: t.ssh.options })).getAllByLabelText(
      texts.components.keyValue.name,
    );
    const second = keys.at(-1);
    if (!second) throw new Error("riga mancante");
    await userEvent.type(second, "ConnectTimeout");

    expect(keys.map((key) => (key as HTMLInputElement).value)).toEqual([
      "ServerAliveCountMax",
      "ConnectTimeout",
    ]);
  });

  it("gives a jump host added after going back and forth a key of its own", async () => {
    unlocked();
    const errors = vi.spyOn(console, "error").mockImplementation(() => undefined);
    renderEditor({ mode: "edit", id: await idOf("db01") });
    const addAddress = async (address: string) => {
      await userEvent.click(screen.getByRole("button", { name: t.ssh.freeAddress }));
      await userEvent.type(screen.getByLabelText(t.ssh.address), `${address}{Enter}`);
    };
    await userEvent.click(await screen.findByRole("button", { name: t.next }));
    await addAddress("jump-a.example.com");
    await userEvent.click(screen.getByRole("button", { name: t.back }));
    await userEvent.click(screen.getByRole("button", { name: t.next }));

    await addAddress("jump-b.example.com");

    const list = screen.getByRole("list", { name: t.ssh.jumps });
    expect(within(list).getByText("jump-a.example.com")).toBeInTheDocument();
    expect(within(list).getByText("jump-b.example.com")).toBeInTheDocument();
    expect(errors.mock.calls.flat().join(" ")).not.toMatch(/same key/);
    errors.mockRestore();
  });

  it("shows a refused ssh option on its row", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("web01") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));

    await userEvent.click(screen.getByRole("button", { name: t.ssh.addOption }));
    const table = screen.getByRole("table", { name: t.ssh.options });
    await userEvent.type(
      within(table).getByLabelText(texts.components.keyValue.name),
      "ProxyCommand",
    );
    await save();

    const row = (await within(table).findByDisplayValue("ProxyCommand")).closest("[role='row']");
    expect(row).toHaveTextContent("l'opzione 'ProxyCommand' non è ammessa");
  });

  it("keeps an imported credential of the gateway through an untouched selector", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    const stored = { mode: "inline", username: "gw-utente", domain: null };
    nodeNamed(state, "ts01").settings = { gateway: { host: "gw.example.com", credential: stored } };
    installFakeBackend(state);
    const { onSaved } = renderEditor({ mode: "edit", id: nodeNamed(state, "ts01").id });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));

    const gateway = screen.getByRole("group", { name: t.rdp.gatewayCredential });
    await userEvent.click(
      within(gateway).getByRole("button", { name: texts.components.credentialField.change }),
    );
    await userEvent.click(
      await screen.findByRole("button", { name: texts.components.credentialSelector.done }),
    );
    await type(t.rdp.wolMac, "00:00:5E:00:53:01");
    await save();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "ts01").settings["gateway"]).toEqual({
      host: "gw.example.com",
      credential: stored,
    });
  });

  it("adds a jump host from the tree", async () => {
    const state = unlocked();
    const { onSaved } = renderEditor({ mode: "edit", id: await idOf("db01") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));

    await userEvent.click(screen.getByRole("button", { name: t.ssh.fromTree }));
    const picker = await screen.findByRole("dialog", { name: texts.dialogs.picker.jumpHost.title });
    await userEvent.click(within(picker).getByRole("option", { name: /bastion/ }));
    await userEvent.click(within(picker).getByRole("button", { name: /^Scegli bastion/ }));
    await save();

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "db01").settings).toEqual({
      jump_hosts: [{ $ref: nodeNamed(state, "bastion").id }],
    });
  });

  it("shows the host of a free IPv6 address in the chain of the jump hosts", async () => {
    unlocked();
    renderEditor({ mode: "edit", id: await idOf("db01") });
    await userEvent.click(await screen.findByRole("button", { name: t.next }));

    await userEvent.click(screen.getByRole("button", { name: t.ssh.freeAddress }));
    await userEvent.type(screen.getByLabelText(t.ssh.address), "admin@[[2001:db8::1]:2222{Enter}");

    const chain = screen.getByRole("list", { name: texts.components.route.label });
    expect(within(chain).getByText("2001:db8::1")).toBeInTheDocument();
  });

  it("renames and moves a node in one save, into a folder with its old name", async () => {
    const { state, id } = withCopyInRete();
    const { onSaved } = renderEditor({ mode: "edit", id });

    await type(t.general.name, "dc02");
    await chooseFolder("Rete");
    await save();

    await waitFor(() =>
      expect(onSaved).toHaveBeenCalledWith(expect.objectContaining({ path: "/Rete/dc02" })),
    );
    expect(nodeNamed(state, "Rete").children.map((child) => child.name)).toEqual([
      "bastion",
      "deploy",
      "dc01",
      "dc02",
    ]);
  });

  it("moves nothing when the save is refused", async () => {
    const { state, id } = withCopyInRete();
    renderEditor({ mode: "edit", id });

    await chooseFolder("Rete");
    await save();

    expect(await screen.findByLabelText(t.general.name)).toHaveAccessibleDescription(
      /esiste già un nodo chiamato 'dc01'/,
    );
    expect(nodeNamed(state, "ACME").children.map((child) => child.name)).toContain("dc01");
  });

  it("keeps the notice of a change from outside when the detail is only read again", async () => {
    unlocked();
    const { client } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.description, "mia descrizione");
    await announceChangeFromOutside();

    // A locked vault or the window coming back read the detail again.
    await act(() => client.invalidateQueries());
    await act(() => client.refetchQueries());

    expect(screen.getByText(t.external.modified)).toBeInTheDocument();
  });

  it("closes the notice of a change from outside once reloaded with the node unchanged", async () => {
    unlocked();
    const { client } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.description, "mia descrizione");
    await announceChangeFromOutside();

    await reloadDocument(client);

    await waitFor(() => expect(screen.queryByText(t.external.modified)).not.toBeInTheDocument());
    expect(screen.queryByText(/Documento ricaricato/)).not.toBeInTheDocument();
    expect(screen.getByLabelText(t.general.description)).toHaveValue("mia descrizione");
  });

  it("lists the fields changed on both sides once reloaded, after other readings", async () => {
    const state = unlocked();
    const { client } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.description, "mia descrizione");
    await announceChangeFromOutside();
    await act(() => client.invalidateQueries());

    const web01 = nodeNamed(state, "web01");
    web01.notes = "nota di un altro programma";
    web01.description = "descrizione di un altro programma";
    await reloadDocument(client);

    expect(await screen.findByText(/Documento ricaricato/)).toHaveTextContent(
      t.general.description,
    );
    expect(screen.getByLabelText(t.general.notes)).toHaveValue("nota di un altro programma");
    expect(screen.getByLabelText(t.general.description)).toHaveValue("mia descrizione");
  });
});
