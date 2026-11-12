import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { SidePanel } from "./SidePanel";
import { TextField } from "./TextField";

describe("SidePanel", () => {
  it("is a modal named by its label with the focus on the first field", () => {
    render(
      <SidePanel open onClose={vi.fn()} label="Modifica connessione SSH">
        <TextField label="Nome" />
      </SidePanel>,
    );

    expect(screen.getByRole("dialog", { name: "Modifica connessione SSH" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Nome" })).toHaveFocus();
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    render(
      <SidePanel open onClose={onClose} label="Nuova cartella">
        <TextField label="Nome" />
      </SidePanel>,
    );

    await userEvent.keyboard("{Escape}");

    expect(onClose).toHaveBeenCalledOnce();
  });

  it("renders nothing when closed", () => {
    render(
      <SidePanel open={false} onClose={vi.fn()} label="Nuova cartella">
        <TextField label="Nome" />
      </SidePanel>,
    );

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("shows control characters of its label", () => {
    render(
      <SidePanel open onClose={vi.fn()} label={"Modifica web\u001b01"}>
        <TextField label="Nome" />
      </SidePanel>,
    );

    expect(screen.getByRole("dialog", { name: "Modifica web\\u{1b}01" })).toBeInTheDocument();
  });
});
