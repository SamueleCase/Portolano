import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { SortableList } from "./SortableList";

interface Jump {
  id: string;
  name: string;
}

function Jumps({ onRemove = vi.fn() }: { onRemove?: (index: number) => void }) {
  const [jumps, setJumps] = useState<Jump[]>([
    { id: "a", name: "bastion" },
    { id: "b", name: "deploy@198.51.100.40:22" },
    { id: "c", name: "gw" },
  ]);
  return (
    <SortableList
      label="Jump host"
      items={jumps}
      itemKey={(jump) => jump.id}
      itemName={(jump) => jump.name}
      renderItem={(jump) => jump.name}
      onMove={(from, to) =>
        setJumps((current) => {
          const next = [...current];
          const [moved] = next.splice(from, 1);
          if (moved) next.splice(to, 0, moved);
          return next;
        })
      }
      onRemove={onRemove}
    />
  );
}

function order(): string[] {
  return screen.getAllByRole("listitem").map((item) => item.textContent ?? "");
}

describe("SortableList", () => {
  it("moves a row down with Alt+↓ and keeps the focus on it", async () => {
    render(<Jumps />);
    const handle = screen.getByRole("button", { name: "Sposta bastion" });
    handle.focus();

    await userEvent.keyboard("{Alt>}{ArrowDown}{/Alt}");

    expect(order()[1]).toContain("bastion");
    expect(screen.getByRole("button", { name: "Sposta bastion" })).toHaveFocus();
  });

  it("announces the new position", async () => {
    render(<Jumps />);
    screen.getByRole("button", { name: "Sposta gw" }).focus();

    await userEvent.keyboard("{Alt>}{ArrowUp}{/Alt}");

    expect(screen.getByText("gw in posizione 2 di 3")).toBeInTheDocument();
  });

  it("does not move past the first row", async () => {
    render(<Jumps />);
    screen.getByRole("button", { name: "Sposta bastion" }).focus();

    await userEvent.keyboard("{Alt>}{ArrowUp}{/Alt}");

    expect(order()[0]).toContain("bastion");
  });

  it("ignores the arrows without Alt", async () => {
    render(<Jumps />);
    screen.getByRole("button", { name: "Sposta bastion" }).focus();

    await userEvent.keyboard("{ArrowDown}");

    expect(order()[0]).toContain("bastion");
  });

  it("explains the keyboard on the handle", () => {
    render(<Jumps />);

    expect(screen.getByRole("button", { name: "Sposta bastion" })).toHaveAccessibleDescription(
      "Alt e freccia su o giù per cambiare l'ordine.",
    );
  });

  it("removes a row with ×", async () => {
    const onRemove = vi.fn();
    render(<Jumps onRemove={onRemove} />);

    await userEvent.click(screen.getByRole("button", { name: "Rimuovi gw" }));

    expect(onRemove).toHaveBeenCalledWith(2);
  });
});
