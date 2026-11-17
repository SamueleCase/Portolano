import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { CredentialChoice_Deserialize } from "../../bindings";
import { texts } from "../../i18n/it";
import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { blankNode } from "../../test/fakeBackendModel";
import { renderWithQueries } from "../../test/render";
import { CredentialChooser } from "./CredentialChooser";

const t = texts.components;

async function renderChooser(value: CredentialChoice_Deserialize = { mode: "inherit" }) {
  const onDone = vi.fn();
  const parent = await idOf("ACME");
  renderWithQueries(
    <CredentialChooser
      open
      nodeName="web09"
      parent={parent}
      value={value}
      passwordState="absent"
      onSetPassword={() => undefined}
      onDone={onDone}
      onClose={() => undefined}
    />,
  );
  return onDone;
}

function preview() {
  return within(screen.getByRole("region", { name: t.credentialPreview.heading }));
}

async function chooseMode(title: string) {
  await userEvent.click(screen.getByRole("radio", { name: new RegExp(`^${title}`) }));
}

const onDone = vi.fn();

describe("CredentialChooser", () => {
  it("shows the chain an inherited credential comes from", async () => {
    unlocked();
    await renderChooser();

    const chain = await preview().findByRole("list", { name: t.credentialPreview.heading });
    expect(
      within(chain)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["web09", "ACME", "Admin ACME"]);
    expect(await preview().findByText("administrator")).toBeInTheDocument();
  });

  it("finds a credential by name ignoring case when no name matches exactly", async () => {
    unlocked();
    await renderChooser({ mode: "byName", name: "admin acme" });

    expect(await preview().findByText("Clienti / Admin ACME")).toBeInTheDocument();
  });

  it("says when no credential has the name", async () => {
    unlocked();
    await renderChooser({ mode: "byName", name: "inesistente" });

    expect(await preview().findByText(t.credentialPreview.byNameNotFound)).toBeInTheDocument();
  });

  it("lists the credentials of an ambiguous name", async () => {
    const state = initialFakeState();
    state.document?.nodes.push(
      Object.assign(blankNode(state, "credential", "deploy"), { username: "altro" }),
    );
    installFakeBackend(state);
    await renderChooser({ mode: "byName", name: "deploy" });

    expect(await preview().findByText(t.credentialPreview.byNameAmbiguous)).toBeInTheDocument();
    expect(preview().getByText("Rete / deploy")).toBeInTheDocument();
    expect(preview().getByText("deploy")).toBeInTheDocument();
  });

  it("changes nothing with Fatto when nothing was touched", async () => {
    unlocked();
    const onClose = vi.fn();
    renderWithQueries(
      <CredentialChooser
        open
        nodeName="gw"
        parent={null}
        value={{ mode: "inherit" }}
        passwordState="absent"
        onSetPassword={() => undefined}
        onDone={onDone}
        onClose={onClose}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: t.credentialSelector.done }));

    expect(onClose).toHaveBeenCalledOnce();
    expect(onDone).not.toHaveBeenCalled();
  });

  it("returns the inline user and domain typed", async () => {
    unlocked();
    const onDone = await renderChooser();
    await chooseMode(t.credentialModes.inline.title);

    await userEvent.type(preview().getByLabelText(t.credentialPreview.user), "operatore");
    await userEvent.click(screen.getByRole("button", { name: t.credentialSelector.done }));

    expect(onDone).toHaveBeenCalledWith({ mode: "inline", username: "operatore", domain: null });
  });

  it("asks for the credential node as soon as Da una credenziale is chosen", async () => {
    unlocked();
    const onDone = await renderChooser();
    await chooseMode(t.credentialModes.linked.title);

    const picker = await screen.findByRole("dialog", {
      name: texts.dialogs.picker.credential.title,
    });
    await userEvent.click(within(picker).getByRole("option", { name: /^deploy/ }));
    await userEvent.click(within(picker).getByRole("button", { name: /^Scegli deploy/ }));
    await userEvent.click(screen.getByRole("button", { name: t.credentialSelector.done }));

    expect(onDone).toHaveBeenCalledWith({ mode: "byId", id: await idOf("deploy") });
  });
});
