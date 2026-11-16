import { screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { idOf } from "../../test/fakeBackendFixtures";
import { importBackend, renderWizard, reportShown, toPasswords } from "./testWizard";

/** Step 4 of the import wizard on the fake backend (inventario §5.13). */

async function importSkippingAll() {
  importBackend();
  const wizard = renderWizard();
  await toPasswords(wizard.user);
  await wizard.user.click(screen.getByRole("button", { name: "Salta tutte le restanti" }));
  await reportShown();
  return wizard;
}

function tile(label: string): HTMLElement {
  const tileLabel = screen.getByText(label, { selector: "span" });
  const tileElement = tileLabel.parentElement;
  if (!tileElement) throw new Error(`tessera ${label} assente`);
  return tileElement;
}

describe("import report", () => {
  it("counts the outcomes of the objects read", async () => {
    await importSkippingAll();

    expect(screen.getByText("elementi letti").parentElement).toHaveTextContent("9");
    expect(tile("Importati")).toHaveTextContent("2");
    expect(tile("Parziali")).toHaveTextContent("4");
    expect(tile("Non supportati")).toHaveTextContent("1");
    expect(tile("Saltati")).toHaveTextContent("2");
  });

  it("makes an item with a skipped password partial and the trash skipped", async () => {
    await importSkippingAll();

    const web10 = screen.getByRole("button", { name: /Ufficio\/Produzione\/web10/ });
    expect(web10).toHaveTextContent("Parziale");
    expect(web10).toHaveTextContent("password Royal cifrata non importata");
    expect(screen.getByText("Ufficio/Cestino").closest("li")).toHaveTextContent("Saltato");
  });

  it("lists an icon over 4 MiB as the reason of a partial item", async () => {
    await importSkippingAll();

    expect(screen.getByRole("button", { name: /dc10/ })).toHaveTextContent(
      "icona personalizzata oltre 4 MiB",
    );
  });

  it("leads from a row to its node through the source id", async () => {
    const { user, onGoToNode } = await importSkippingAll();

    await user.click(screen.getByRole("button", { name: /Ufficio\/Produzione\/web10/ }));

    expect(onGoToNode).toHaveBeenCalledWith(await idOf("web10"));
  });

  it("does not lead anywhere from an item of the trash", async () => {
    await importSkippingAll();

    const trash = screen.getByText("Ufficio/Cestino").closest("li");
    expect(trash && within(trash).queryByRole("button")).toBeNull();
  });

  it("filters the unsupported items", async () => {
    const { user } = await importSkippingAll();

    await user.click(screen.getByRole("button", { name: "Non supportati" }));

    const list = screen.getByText(/^Elementi da controllare/).closest("section");
    expect(list && within(list).getAllByRole("listitem")).toHaveLength(1);
  });

  it("exports the report as JSON with the native dialog", async () => {
    const { user } = await importSkippingAll();

    await user.click(screen.getByRole("button", { name: "Esporta JSON" }));

    expect(await screen.findByText("Report salvato in report-import.json.")).toBeInTheDocument();
  });

  it("goes to the imported folder and closes with Vai all'albero", async () => {
    const { user, onGoToNode, onClose } = await importSkippingAll();

    await user.click(screen.getByRole("button", { name: "Vai all'albero" }));

    expect(onGoToNode).toHaveBeenCalledWith(await idOf("Ufficio"));
    expect(onClose).toHaveBeenCalled();
  });
});
