import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { ColorPicker } from "./ColorPicker";

function Picker({
  initial = null,
  onChange,
}: {
  initial?: string | null;
  onChange?: (value: string | null) => void;
}) {
  const [value, setValue] = useState<string | null>(initial);
  return (
    <ColorPicker
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
    />
  );
}

describe("ColorPicker", () => {
  it("offers six colors, none and a custom one", () => {
    render(<Picker />);

    expect(screen.getByRole("radiogroup", { name: "Colore" })).toBeInTheDocument();
    expect(screen.getAllByRole("radio")).toHaveLength(8);
    expect(screen.getByRole("radio", { name: "Nessuno" })).toBeChecked();
  });

  it("chooses a swatch", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);

    await userEvent.click(screen.getByRole("radio", { name: "Terracotta" }));

    expect(onChange).toHaveBeenCalledWith("#D98F7F");
    expect(screen.getByRole("radio", { name: "Terracotta" })).toBeChecked();
  });

  it("accepts a custom hexadecimal color", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);

    await userEvent.click(screen.getByRole("radio", { name: "Personalizzato…" }));
    await userEvent.type(screen.getByRole("textbox", { name: "Colore esadecimale" }), "#123abc");

    expect(onChange).toHaveBeenLastCalledWith("#123abc");
  });

  it("says how to write an invalid color", async () => {
    render(<Picker />);

    await userEvent.click(screen.getByRole("radio", { name: "Personalizzato…" }));
    await userEvent.type(screen.getByRole("textbox", { name: "Colore esadecimale" }), "rosso");

    expect(screen.getByRole("alert")).toHaveTextContent("Scrivi il colore come #RRGGBB.");
  });

  it("shows an imported color outside the six as the custom one", () => {
    render(<Picker initial="#102030" />);

    expect(screen.getByRole("radio", { name: "Personalizzato…" })).toBeChecked();
    expect(screen.getByRole("textbox", { name: "Colore esadecimale" })).toHaveValue("#102030");
  });

  it("removes the color with Nessuno", async () => {
    const onChange = vi.fn();
    render(<Picker initial="#7FC2A0" onChange={onChange} />);

    await userEvent.click(screen.getByRole("radio", { name: "Nessuno" }));

    expect(onChange).toHaveBeenCalledWith(null);
  });
});
