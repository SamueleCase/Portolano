import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Toggle } from "./Toggle";
import { ToggleRow } from "./ToggleRow";

describe("Toggle", () => {
  it("is a switch that reports the opposite state when clicked", async () => {
    const onChange = vi.fn();
    render(<Toggle label="Compressione" checked={false} onChange={onChange} />);
    const toggle = screen.getByRole("switch", { name: "Compressione" });

    await userEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-checked", "false");
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("can be switched with the space bar", async () => {
    const onChange = vi.fn();
    render(<Toggle label="Compressione" checked onChange={onChange} />);

    await userEvent.tab();
    await userEvent.keyboard(" ");

    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("ignores clicks when disabled", async () => {
    const onChange = vi.fn();
    render(<Toggle label="X11 trusted" checked={false} disabled onChange={onChange} />);

    await userEvent.click(screen.getByRole("switch"));

    expect(onChange).not.toHaveBeenCalled();
  });
});

describe("ToggleRow", () => {
  it("names the switch with its label and describes it with its description", () => {
    render(
      <ToggleRow
        label="Riduci movimento"
        description="Ferma lo sfondo e sostituisce gli spostamenti con dissolvenze."
        checked
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("switch", { name: "Riduci movimento" })).toHaveAccessibleDescription(
      "Ferma lo sfondo e sostituisce gli spostamenti con dissolvenze.",
    );
  });
});
