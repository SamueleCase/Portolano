import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";

import { type Segment, SegmentedControl } from "./SegmentedControl";

type Theme = "dark" | "light" | "system";

const themes: Segment<Theme>[] = [
  { value: "dark", label: "Scuro" },
  { value: "light", label: "Chiaro · in arrivo", unavailable: true },
  { value: "system", label: "Sistema" },
];

function Themes() {
  const [value, setValue] = useState<Theme>("dark");
  return <SegmentedControl label="Tema" options={themes} value={value} onChange={setValue} />;
}

describe("SegmentedControl", () => {
  it("is a radio group with the chosen option checked", () => {
    render(<Themes />);

    expect(screen.getByRole("radiogroup", { name: "Tema" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Scuro" })).toBeChecked();
  });

  it("is a single tab stop on the chosen option", async () => {
    render(<Themes />);

    await userEvent.tab();

    expect(screen.getByRole("radio", { name: "Scuro" })).toHaveFocus();
  });

  it("chooses the next available option with the arrows", async () => {
    render(<Themes />);

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}");

    expect(screen.getByRole("radio", { name: "Sistema" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "Sistema" })).toHaveFocus();
  });

  it("does not choose an unavailable option", async () => {
    render(<Themes />);

    await userEvent.click(screen.getByRole("radio", { name: "Chiaro · in arrivo" }));

    expect(screen.getByRole("radio", { name: "Scuro" })).toBeChecked();
  });
});
