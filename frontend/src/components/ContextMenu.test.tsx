import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ContextMenu, type MenuEntry } from "./ContextMenu";

const entries: MenuEntry[] = [
  { id: "connect", label: "Connetti", shortcut: "⌘↩" },
  { id: "external", label: "Apri in terminale esterno" },
  { separator: true },
  { id: "edit", label: "Modifica…", shortcut: "⌘E", disabled: true },
  { id: "delete", label: "Elimina…", shortcut: "⌘⌫", destructive: true },
];

function renderMenu(onSelect = vi.fn(), onClose = vi.fn()) {
  render(
    <ContextMenu
      label="web01"
      entries={entries}
      position={{ x: 40, y: 60 }}
      onSelect={onSelect}
      onClose={onClose}
    />,
  );
  return { onSelect, onClose };
}

describe("ContextMenu", () => {
  it("is a menu with the focus on its first entry", () => {
    renderMenu();

    expect(screen.getByRole("menu", { name: "web01" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Connetti" })).toHaveFocus();
  });

  it("skips disabled entries with the arrows and wraps around", async () => {
    renderMenu();

    await userEvent.keyboard("{ArrowDown}{ArrowDown}");
    expect(screen.getByRole("menuitem", { name: "Elimina…" })).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}");

    expect(screen.getByRole("menuitem", { name: "Connetti" })).toHaveFocus();
  });

  it("chooses the focused entry with Enter and closes", async () => {
    const { onSelect, onClose } = renderMenu();

    await userEvent.keyboard("{End}{Enter}");

    expect(onSelect).toHaveBeenCalledWith("delete");
    expect(onClose).toHaveBeenCalled();
  });

  it("closes on Escape", async () => {
    const { onClose } = renderMenu();

    await userEvent.keyboard("{Escape}");

    expect(onClose).toHaveBeenCalledOnce();
  });

  it("closes on a click outside", async () => {
    const { onClose } = renderMenu();

    await userEvent.click(document.body);

    expect(onClose).toHaveBeenCalledOnce();
  });
});
