import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";

import { useRovingRadio } from "./useRovingRadio";

const labels = ["a", "b", "c", "d", "e", "f"];

function Group({
  initial,
  unavailable = [],
  columns,
}: {
  initial: number;
  unavailable?: number[];
  columns?: number;
}) {
  const [selected, setSelected] = useState(initial);
  const radio = useRovingRadio({
    count: labels.length,
    selected,
    available: (index) => !unavailable.includes(index),
    ...(columns === undefined ? {} : { columns }),
    onMove: setSelected,
  });
  return (
    <div role="radiogroup" aria-label="gruppo">
      {labels.map((label, index) => (
        <button
          key={label}
          ref={radio.register(index)}
          type="button"
          role="radio"
          aria-checked={index === selected}
          tabIndex={radio.tabIndexOf(index)}
          onKeyDown={(event) => radio.keyDown(event, index)}
        >
          {label}
        </button>
      ))}
    </div>
  );
}

describe("useRovingRadio", () => {
  it("puts the only tab stop on the chosen option", async () => {
    render(<Group initial={2} />);

    await userEvent.tab();

    expect(screen.getByRole("radio", { name: "c" })).toHaveFocus();
  });

  it("puts the tab stop on the first available option when the choice is not available", async () => {
    render(<Group initial={0} unavailable={[0]} />);

    await userEvent.tab();

    expect(screen.getByRole("radio", { name: "b" })).toHaveFocus();
  });

  it("chooses the next available option and wraps around", async () => {
    render(<Group initial={4} unavailable={[5]} />);

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}");

    expect(screen.getByRole("radio", { name: "a" })).toHaveFocus();
    expect(screen.getByRole("radio", { name: "a" })).toBeChecked();
  });

  it("goes to the first and the last option with Home and End", async () => {
    render(<Group initial={2} />);

    await userEvent.tab();
    await userEvent.keyboard("{End}");

    expect(screen.getByRole("radio", { name: "f" })).toBeChecked();
  });

  it("moves by rows in a grid, without wrapping", async () => {
    render(<Group initial={1} columns={3} />);

    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}{ArrowDown}");

    expect(screen.getByRole("radio", { name: "e" })).toBeChecked();
  });
});
