import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { FilterChip } from "./FilterChip";

describe("FilterChip", () => {
  it("announces whether it is on", () => {
    render(
      <FilterChip active onToggle={vi.fn()}>
        SSH
      </FilterChip>,
    );

    expect(screen.getByRole("button", { name: "SSH" })).toHaveAttribute("aria-pressed", "true");
  });

  it("gives a symbol a name", async () => {
    const onToggle = vi.fn();
    render(
      <FilterChip active={false} onToggle={onToggle} label="Preferiti">
        ★
      </FilterChip>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Preferiti" }));

    expect(onToggle).toHaveBeenCalledOnce();
  });
});
