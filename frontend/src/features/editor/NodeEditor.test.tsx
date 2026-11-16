import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { emit } from "@tauri-apps/api/event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { createQueryClient } from "../../ipc/queryClient";
import {
  type FakeBackendState,
  TEST_MASTER_PASSWORD,
  initialFakeState,
  installFakeBackend,
} from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { type FakeNode, blankNode, walk } from "../../test/fakeBackendModel";
import { stubScrollIntoView } from "../dialogs/scrollStub";
import { type EditorRequest, NodeEditor } from "./NodeEditor";

stubScrollIntoView();

const t = texts.editor;
const secret = texts.dialogs.secret;
/** A sentinel secret: it must never be found in the page or in the cache of Query. */
const SENTINEL = "sentinel-editor-secret-3fa9";

function nodeNamed(state: FakeBackendState, name: string): FakeNode {
  const found = [...walk(state.document?.nodes ?? [])].find(({ node }) => node.name === name);
  if (!found) throw new Error(`nodo ${name} assente`);
  return found.node;
}

function renderEditor(request: EditorRequest) {
  const client = createQueryClient();
  const onSaved = vi.fn();
  const onClose = vi.fn();
  render(
    <QueryClientProvider client={client}>
      <NodeEditor request={request} onClose={onClose} onSaved={onSaved} />
    </QueryClientProvider>,
  );
  return { client, onSaved, onClose };
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
    await userEvent.type(
      await screen.findByLabelText(texts.dialogs.unlock.field),
      TEST_MASTER_PASSWORD,
    );
    await userEvent.click(screen.getByRole("button", { name: texts.dialogs.unlock.unlock }));

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(nodeNamed(state, "deploy").secrets["password"]).toBe(true);
    expect(document.body.innerHTML).not.toContain(SENTINEL);
    expect(cacheText(client)).not.toContain(SENTINEL);
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
    expect(document.body.innerHTML).not.toContain(SENTINEL);
    expect(cacheText(client)).not.toContain(SENTINEL);
    await type(t.general.name, "deploy-2");
    await save();
    await waitFor(() => expect(nodeNamed(state, "deploy-2")).toBeDefined());
    expect(nodeNamed(state, "deploy-2").secrets["password"]).toBeUndefined();
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

  it("keeps its changes over a document reloaded after a change from outside", async () => {
    const state = unlocked();
    const { client } = renderEditor({ mode: "edit", id: await idOf("web01") });
    await type(t.general.description, "mia descrizione");
    await act(() => emit("document-modified-externally", { fileName: "portolano.json" }));
    expect(await screen.findByText(t.external.modified)).toBeInTheDocument();

    const web01 = nodeNamed(state, "web01");
    web01.notes = "nota di un altro programma";
    web01.description = "descrizione di un altro programma";
    await act(() => client.invalidateQueries());

    expect(await screen.findByText(/Documento ricaricato/)).toHaveTextContent(
      t.general.description,
    );
    expect(screen.getByLabelText(t.general.notes)).toHaveValue("nota di un altro programma");
    expect(screen.getByLabelText(t.general.description)).toHaveValue("mia descrizione");
  });
});
