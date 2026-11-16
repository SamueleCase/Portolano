import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";

import { NodePicker, type PickerNode } from "./NodePicker";

// jsdom does not lay out, so it has no `scrollIntoView`.
const scrollIntoView = vi.fn();
beforeAll(() => {
  Element.prototype.scrollIntoView = scrollIntoView;
});
afterAll(() => {
  Reflect.deleteProperty(Element.prototype, "scrollIntoView");
});

const nodes: PickerNode[] = [
  { id: "prod", name: "Produzione", kind: "folder", depth: 0 },
  { id: "bastion", name: "bastion", kind: "ssh", depth: 1, host: "bastion.example.com" },
  { id: "web01", name: "web01", kind: "ssh", depth: 1, host: "web01.example.com", excluded: true },
  { id: "stg", name: "Staging", kind: "folder", depth: 0 },
  { id: "stg-bastion", name: "stg-bastion", kind: "ssh", depth: 1, host: "192.0.2.30" },
];

function renderPicker(onChoose = vi.fn()) {
  render(
    <NodePicker
      open
      onClose={vi.fn()}
      onChoose={onChoose}
      title="Scegli un jump host"
      filterLabel="Solo SSH"
      nodes={nodes}
      note="web01 è escluso perché non può fare da salto a se stesso."
    />,
  );
  return onChoose;
}

describe("NodePicker", () => {
  it("starts with the focus on the search", () => {
    renderPicker();

    expect(screen.getByRole("combobox", { name: "Cerca…" })).toHaveFocus();
  });

  it("filters the tree while typing and keeps the folders of the results", async () => {
    renderPicker();

    await userEvent.keyboard("stg");

    expect(screen.getAllByRole("option")).toHaveLength(1);
    expect(screen.getByText("Staging")).toBeInTheDocument();
    expect(screen.queryByText("Produzione")).not.toBeInTheDocument();
  });

  it("names the chosen node on the confirm button", () => {
    renderPicker();

    expect(screen.getByRole("button", { name: "Scegli bastion" })).toBeInTheDocument();
  });

  it("skips excluded nodes with the arrows and chooses with Enter", async () => {
    const onChoose = renderPicker();

    await userEvent.keyboard("{ArrowDown}{Enter}");

    expect(onChoose).toHaveBeenCalledWith("stg-bastion");
  });

  it("counts the results and explains the exclusions", () => {
    renderPicker();

    expect(screen.getByRole("status")).toHaveTextContent(
      "2 risultati. web01 è escluso perché non può fare da salto a se stesso.",
    );
  });

  it("says when nothing matches", async () => {
    renderPicker();

    await userEvent.keyboard("zzz");

    expect(screen.getByRole("status")).toHaveTextContent("Nessun nodo corrisponde alla ricerca.");
  });

  it("describes each node with the folders that contain it", () => {
    renderPicker();

    expect(screen.getByRole("option", { name: /^bastion/ })).toHaveAccessibleDescription(
      "Radice del documento / Produzione",
    );
  });
});

const tree: PickerNode[] = [
  { id: "prod", name: "Produzione", kind: "folder", depth: 0 },
  { id: "bastion", name: "bastion", kind: "ssh", depth: 1, host: "bastion.example.com" },
  { id: "db", name: "Database", kind: "folder", depth: 1 },
  { id: "archive", name: "Archivio", kind: "folder", depth: 0 },
  { id: "old", name: "Vecchi", kind: "folder", depth: 1 },
  { id: "old-gw", name: "old-gw", kind: "ssh", depth: 2, host: "198.51.100.7" },
  { id: "stg", name: "Staging", kind: "folder", depth: 0 },
];

function renderFolderPicker(exclude?: string, onChoose = vi.fn()) {
  render(
    <NodePicker
      open
      onClose={vi.fn()}
      onChoose={onChoose}
      choose="folders"
      exclude={exclude}
      title="Sposta in…"
      filterLabel="Solo cartelle"
      nodes={tree}
    />,
  );
  return onChoose;
}

function optionNames() {
  return screen.getAllByRole("option").map((option) => option.textContent);
}

describe("NodePicker choosing folders", () => {
  it("offers the root of the document and every folder, empty ones included", () => {
    renderFolderPicker();

    expect(optionNames()).toEqual([
      "Radice del documento",
      "Produzione",
      "Database",
      "Archivio",
      "Vecchi",
      "Staging",
    ]);
  });

  it("leaves out the connections", () => {
    renderFolderPicker();

    expect(screen.queryByText("bastion")).not.toBeInTheDocument();
  });

  it("starts on the root and chooses it as null with Enter", async () => {
    const onChoose = renderFolderPicker();

    await userEvent.keyboard("{Enter}");

    expect(onChoose).toHaveBeenCalledWith(null);
  });

  it("names the root on the confirm button", () => {
    renderFolderPicker();

    expect(
      screen.getByRole("button", { name: "Scegli la radice del documento" }),
    ).toBeInTheDocument();
  });

  it("chooses an empty folder with the arrows", async () => {
    const onChoose = renderFolderPicker();

    await userEvent.keyboard("{ArrowDown}{ArrowDown}{Enter}");

    expect(onChoose).toHaveBeenCalledWith("db");
  });

  it("disables the excluded folder and its descendants", () => {
    renderFolderPicker("archive");

    const disabled = screen
      .getAllByRole("option")
      .filter((option) => option.getAttribute("aria-disabled") === "true")
      .map((option) => option.textContent);
    expect(disabled).toEqual(["Archivio", "Vecchi"]);
  });

  it("skips the excluded subtree with the arrows", async () => {
    const onChoose = renderFolderPicker("archive");

    await userEvent.keyboard("{ArrowDown}{ArrowDown}{ArrowDown}{Enter}");

    expect(onChoose).toHaveBeenCalledWith("stg");
  });

  it("does not choose an excluded folder on double click", async () => {
    const onChoose = renderFolderPicker("archive");

    await userEvent.dblClick(screen.getByRole("option", { name: "Vecchi" }));

    expect(onChoose).not.toHaveBeenCalled();
  });

  it("filters the folders while typing and keeps the folders that lead to them", async () => {
    renderFolderPicker();

    await userEvent.keyboard("data");

    expect(optionNames()).toEqual(["Produzione", "Database"]);
  });

  it("starts on the folder found, not on the folder that leads to it", async () => {
    const onChoose = renderFolderPicker();

    await userEvent.keyboard("data{Enter}");

    expect(onChoose).toHaveBeenCalledWith("db");
  });

  it("reaches the folder that leads to a result with the arrows", async () => {
    const onChoose = renderFolderPicker();

    await userEvent.keyboard("data{ArrowUp}{Enter}");

    expect(onChoose).toHaveBeenCalledWith("prod");
  });

  it("counts only the folders found", async () => {
    renderFolderPicker();

    await userEvent.keyboard("data");

    expect(screen.getByRole("status")).toHaveTextContent("1 risultato.");
  });

  it("says when only excluded folders match", async () => {
    renderFolderPicker("archive");

    await userEvent.keyboard("vecchi");

    expect(screen.getByRole("status")).toHaveTextContent(
      "Alla ricerca corrispondono solo voci escluse.",
    );
  });

  it("finds the root by its name", async () => {
    renderFolderPicker();

    await userEvent.keyboard("radice");

    expect(optionNames()).toEqual(["Radice del documento"]);
  });

  it("describes each folder with the folders that contain it", () => {
    renderFolderPicker();

    expect(screen.getByRole("option", { name: "Vecchi" })).toHaveAccessibleDescription(
      "Radice del documento / Archivio",
    );
  });

  it("describes the folder on the confirm button with its path", async () => {
    renderFolderPicker();

    await userEvent.keyboard("{ArrowDown}{ArrowDown}");

    expect(screen.getByRole("button", { name: "Scegli Database" })).toHaveAccessibleDescription(
      "Radice del documento / Produzione",
    );
  });

  it("gives the root no path", () => {
    renderFolderPicker();

    expect(screen.getByRole("option", { name: "Radice del documento" })).not.toHaveAttribute(
      "aria-describedby",
    );
  });

  it("scrolls the active folder into view", async () => {
    renderFolderPicker();

    await userEvent.keyboard("{ArrowDown}");

    expect(scrollIntoView).toHaveBeenLastCalledWith({ block: "nearest" });
    expect(scrollIntoView.mock.contexts.at(-1)).toBe(
      screen.getByRole("option", { name: "Produzione" }),
    );
  });

  it("says when no folder matches", async () => {
    renderFolderPicker();

    await userEvent.keyboard("zzz");

    expect(screen.getByRole("status")).toHaveTextContent(
      "Nessuna cartella corrisponde alla ricerca.",
    );
  });
});
