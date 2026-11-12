import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { TextArea } from "./TextArea";
import { TextField } from "./TextField";

describe("TextField", () => {
  it("is named by its label", async () => {
    const onChange = vi.fn();
    render(<TextField label="Host" value="" onChange={onChange} />);

    await userEvent.type(screen.getByRole("textbox", { name: "Host" }), "a");

    expect(onChange).toHaveBeenCalled();
  });

  it("marks an error as invalid and describes the field with it", () => {
    render(
      <TextField
        label="Porta"
        defaultValue="70000"
        message={{ tone: "error", text: "La porta va da 1 a 65535" }}
      />,
    );
    const field = screen.getByRole("textbox", { name: "Porta" });

    expect(field).toHaveAttribute("aria-invalid", "true");
    expect(field).toHaveAccessibleDescription("La porta va da 1 a 65535");
  });

  it("describes a confirmed field without marking it invalid", () => {
    render(
      <TextField
        label="Conferma"
        type="password"
        message={{ tone: "ok", text: "Le due password coincidono" }}
      />,
    );
    const field = screen.getByLabelText("Conferma");

    expect(field).not.toHaveAttribute("aria-invalid");
    expect(field).toHaveAccessibleDescription("Le due password coincidono");
  });

  it("keeps the label of a search field for assistive technologies", () => {
    render(<TextField label="Cerca" variant="search" placeholder="Cerca per nome, host, tag…" />);

    expect(screen.getByRole("textbox", { name: "Cerca" })).toBeInTheDocument();
  });
});

describe("TextArea", () => {
  it("is named by its label", () => {
    render(<TextArea label="Note" defaultValue="Replica in sola lettura." />);

    expect(screen.getByRole("textbox", { name: "Note" })).toHaveValue("Replica in sola lettura.");
  });
});
