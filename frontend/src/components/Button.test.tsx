import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../i18n/it";
import { Button } from "./Button";

describe("Button", () => {
  it("runs its action when clicked", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Salva</Button>);

    await userEvent.click(screen.getByRole("button", { name: "Salva" }));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it("is a plain button, never a submit", () => {
    render(<Button>Annulla</Button>);

    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
  });

  it("stays focusable but inert when disabled (ambiguity 7)", async () => {
    const onClick = vi.fn();
    render(
      <Button variant="primary" disabled onClick={onClick}>
        Salva
      </Button>,
    );
    const button = screen.getByRole("button", { name: "Salva" });

    await userEvent.tab();
    await userEvent.click(button);

    expect(button).toHaveFocus();
    expect(button).toHaveAttribute("aria-disabled", "true");
    expect(onClick).not.toHaveBeenCalled();
  });

  it("keeps its label, says it is busy and ignores clicks while loading", async () => {
    const onClick = vi.fn();
    render(
      <Button loading onClick={onClick}>
        Sblocca
      </Button>,
    );
    const button = screen.getByRole("button", { name: `Sblocca ${texts.components.busy}` });

    await userEvent.click(button);

    expect(button).toHaveAttribute("aria-busy", "true");
    expect(onClick).not.toHaveBeenCalled();
  });
});
