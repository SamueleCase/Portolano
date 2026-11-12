import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ModeChip } from "./ModeChip";
import { SourceChip } from "./SourceChip";
import { Tag } from "./Tag";

describe("Tag", () => {
  it("shows control characters of the document visibly", () => {
    render(<Tag name={"web\u001b[2J"} />);

    expect(screen.getByText("web\\u{1b}[2J")).toBeInTheDocument();
  });

  it("can be removed in the editor", async () => {
    const onRemove = vi.fn();
    render(<Tag name="postgres" variant="editor" onRemove={onRemove} />);

    await userEvent.click(screen.getByRole("button", { name: "Rimuovi il tag postgres" }));

    expect(onRemove).toHaveBeenCalledOnce();
  });
});

describe("ModeChip", () => {
  it("shows control characters of a folder name visibly", () => {
    render(<ModeChip>{"Ereditata dalla cartella Prod\u0007"}</ModeChip>);

    expect(screen.getByText("Ereditata dalla cartella Prod\\u{7}")).toBeInTheDocument();
  });
});

describe("SourceChip", () => {
  it("names where a jump host comes from", () => {
    render(<SourceChip source="address" />);

    expect(screen.getByText("indirizzo libero")).toBeInTheDocument();
  });
});
