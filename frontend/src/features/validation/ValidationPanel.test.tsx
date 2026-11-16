import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { initialFakeState, installFakeBackend } from "../../test/fakeBackend";
import { idOf, unlocked } from "../../test/fakeBackendFixtures";
import { renderWithQueries } from "../../test/render";
import { ValidationPanel } from "./ValidationPanel";

/** The validation tab on the fake backend (inventario §5.14). */

function rows() {
  return within(screen.getByRole("list", { name: "Problemi" })).getAllByRole("listitem");
}

describe("ValidationPanel", () => {
  it("counts the problems of the document per level", async () => {
    unlocked();

    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);

    expect(
      await screen.findByRole("heading", { name: "2 problemi nel documento" }),
    ).toBeInTheDocument();
    const counts = screen.getByRole("list", { name: "Problemi per gravità" });
    expect(
      within(counts)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["1 avviso", "1 nota"]);
  });

  it("lists the rows with the readable path of their node", async () => {
    unlocked();

    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);

    await screen.findByRole("heading", { name: "2 problemi nel documento" });
    expect(rows()[0]).toHaveTextContent("Rete / deploy");
  });

  it("leads to the node of a row", async () => {
    unlocked();
    const onGoToNode = vi.fn();
    renderWithQueries(<ValidationPanel onGoToNode={onGoToNode} />);
    await screen.findByRole("heading", { name: "2 problemi nel documento" });

    await userEvent.click(within(rows()[1] as HTMLElement).getByRole("button"));

    expect(onGoToNode).toHaveBeenCalledWith(await idOf("vnc-legacy"));
  });

  it("shows the note of the locked vault without a way to a node", async () => {
    installFakeBackend();

    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);

    await screen.findByRole("heading", { name: "2 problemi nel documento" });
    const note = rows().find((row) => row.textContent?.includes("il vault è bloccato"));
    expect(note && within(note).queryByRole("button")).toBeNull();
  });

  it("says when the document has no problem", async () => {
    const state = unlocked();
    if (state.document) state.document.nodes = [];

    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);

    expect(
      await screen.findByRole("heading", { name: "Nessun problema nel documento" }),
    ).toBeInTheDocument();
  });

  it("checks again with Ricontrolla", async () => {
    const state = unlocked();
    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);
    await screen.findByRole("heading", { name: "2 problemi nel documento" });
    if (state.document) state.document.nodes = [];

    await userEvent.click(screen.getByRole("button", { name: "Ricontrolla" }));

    expect(
      await screen.findByRole("heading", { name: "Nessun problema nel documento" }),
    ).toBeInTheDocument();
  });

  it("shows the control characters of a message instead of running them", async () => {
    const state = initialFakeState();
    state.vault.locked = false;
    const vnc = state.document?.nodes.find((node) => node.name === "vnc-legacy");
    if (vnc) vnc.name = "vnc\u001b[31m";
    installFakeBackend(state);

    renderWithQueries(<ValidationPanel onGoToNode={() => undefined} />);

    await screen.findByRole("heading", { name: "2 problemi nel documento" });
    expect(rows()[1]).toHaveTextContent("vnc\\u{1b}[31m");
  });
});
