import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { Select } from "./Select";

const depths = [
  { value: "16", label: "16 bit" },
  { value: "24", label: "24 bit" },
  { value: "32", label: "32 bit" },
] as const;

function Depth({ onChange }: { onChange?: (value: string) => void }) {
  const [value, setValue] = useState<"16" | "24" | "32">("32");
  return (
    <Select
      label="Profondità colore"
      options={depths}
      value={value}
      onChange={(next) => {
        setValue(next);
        onChange?.(next);
      }}
    />
  );
}

describe("Select", () => {
  it("shows the chosen option in the closed field", () => {
    render(<Depth />);

    expect(screen.getByRole("combobox", { name: "Profondità colore" })).toHaveTextContent("32 bit");
  });

  it("opens the list on click and chooses an option", async () => {
    const onChange = vi.fn();
    render(<Depth onChange={onChange} />);

    await userEvent.click(screen.getByRole("combobox"));
    await userEvent.click(screen.getByRole("option", { name: "16 bit" }));

    expect(onChange).toHaveBeenCalledWith("16");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });

  it("chooses with the arrows and Enter, keeping the focus on the field", async () => {
    render(<Depth />);
    const field = screen.getByRole("combobox");

    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}{ArrowUp}{Enter}");

    expect(field).toHaveFocus();
    expect(field).toHaveTextContent("24 bit");
  });

  it("closes without choosing on Escape", async () => {
    render(<Depth />);

    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}{Home}{Escape}");

    expect(screen.getByRole("combobox")).toHaveTextContent("32 bit");
    expect(screen.getByRole("combobox")).toHaveAttribute("aria-expanded", "false");
  });
});
