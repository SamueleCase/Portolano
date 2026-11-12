import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { IconPicker } from "./IconPicker";

const hash = "ab".repeat(32);

function renderPicker(onUse = vi.fn(), onChooseImage = vi.fn()) {
  render(
    <IconPicker
      open
      onClose={vi.fn()}
      value={undefined}
      onUse={onUse}
      customIcons={[
        { fileName: `${hash}.png`, label: "Server Royal" },
        { fileName: "../escape.png", label: "Non valida" },
      ]}
      onChooseImage={onChooseImage}
    />,
  );
  return { onUse, onChooseImage };
}

describe("IconPicker", () => {
  it("offers every builtin icon with its Italian name", () => {
    renderPicker();

    expect(screen.getAllByRole("radio", { name: /./ }).length).toBeGreaterThanOrEqual(30);
    expect(screen.getByRole("radio", { name: "Disco" })).toBeInTheDocument();
  });

  it("filters the icons by their Italian name", async () => {
    renderPicker();

    await userEvent.type(screen.getByRole("textbox", { name: "Cerca icona…" }), "stamp");

    expect(screen.getByRole("radio", { name: "Stampante" })).toBeInTheDocument();
    expect(screen.queryByRole("radio", { name: "Server" })).not.toBeInTheDocument();
  });

  it("uses the chosen builtin icon", async () => {
    const { onUse } = renderPicker();

    await userEvent.click(screen.getByRole("radio", { name: "Database" }));
    await userEvent.click(screen.getByRole("button", { name: "Usa icona" }));

    expect(onUse).toHaveBeenCalledWith({ source: "builtin", name: "database" });
  });

  it("does not use an icon before one is chosen", async () => {
    const { onUse } = renderPicker();

    await userEvent.click(screen.getByRole("button", { name: "Usa icona" }));

    expect(onUse).not.toHaveBeenCalled();
  });

  it("shows imported icons only as images from the icon scheme, skipping invalid names", async () => {
    renderPicker();

    await userEvent.click(screen.getByRole("radio", { name: "Importate" }));

    const tiles = screen.getAllByRole("radio", { name: /^Icona importata/ });
    expect(tiles).toHaveLength(1);
    expect(tiles[0]?.querySelector("img")).toHaveAttribute(
      "src",
      `portolano-icon://localhost/${hash}.png`,
    );
  });

  it("asks Rust for an image from disk", async () => {
    const { onChooseImage } = renderPicker();

    await userEvent.click(screen.getByRole("radio", { name: "Immagine" }));
    await userEvent.click(screen.getByRole("button", { name: "Scegli un'immagine…" }));

    expect(onChooseImage).toHaveBeenCalledOnce();
  });

  it("starts again from the node's icon, the builtin source and an empty search", async () => {
    function Reopen() {
      const [open, setOpen] = useState(true);
      return (
        <>
          <button type="button" onClick={() => setOpen(true)}>
            riapri
          </button>
          <IconPicker
            open={open}
            onClose={() => setOpen(false)}
            value={undefined}
            onUse={vi.fn()}
            customIcons={[]}
            onChooseImage={vi.fn()}
          />
        </>
      );
    }
    render(<Reopen />);
    await userEvent.type(screen.getByRole("textbox", { name: "Cerca icona…" }), "stamp");
    await userEvent.click(screen.getByRole("radio", { name: "Stampante" }));
    await userEvent.click(screen.getByRole("button", { name: "Annulla" }));

    await userEvent.click(screen.getByRole("button", { name: "riapri" }));

    expect(screen.getByRole("textbox", { name: "Cerca icona…" })).toHaveValue("");
    expect(screen.getByRole("radio", { name: "Portolano" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "Stampante" })).not.toBeChecked();
  });
});
