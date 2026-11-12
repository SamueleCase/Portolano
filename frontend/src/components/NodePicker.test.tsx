import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { NodePicker, type PickerNode } from "./NodePicker";

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
});
