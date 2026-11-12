import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Button } from "./Button";
import { CommandPreview } from "./CommandPreview";
import { DropZone } from "./DropZone";
import { EmptyState } from "./EmptyState";
import { GroupHeader } from "./GroupHeader";
import { IconTile } from "./IconTile";
import { KeyValueGrid } from "./KeyValueGrid";
import { Logo } from "./Logo";

describe("EmptyState", () => {
  it("is a region named by its title", () => {
    render(
      <EmptyState
        title="Il documento è vuoto"
        text="Aggiungi la prima connessione."
        actions={<Button>Nuova connessione</Button>}
      />,
    );

    expect(screen.getByRole("region", { name: "Il documento è vuoto" })).toBeInTheDocument();
  });
});

describe("KeyValueGrid", () => {
  it("pairs each label with its value", () => {
    render(<KeyValueGrid rows={[{ key: "client", label: "Client", value: "FreeRDP" }]} />);

    expect(screen.getByRole("term")).toHaveTextContent("Client");
    expect(screen.getByRole("definition")).toHaveTextContent("FreeRDP");
  });
});

describe("CommandPreview", () => {
  it("shows the command of the plan as text, control characters visible", () => {
    render(<CommandPreview caption="Comando che verrà lanciato." command={"ssh -- web\u001b01"} />);

    expect(screen.getByText("ssh -- web\\u{1b}01")).toBeInTheDocument();
  });
});

describe("DropZone", () => {
  it("reports a drop", () => {
    const onDrop = vi.fn();
    render(
      <DropZone
        title="Trascina qui"
        hint="File .rtsz, oppure"
        action={<Button>Scegli file…</Button>}
        onDrop={onDrop}
      />,
    );

    fireEvent.drop(screen.getByText("Trascina qui"));

    expect(onDrop).toHaveBeenCalledOnce();
  });
});

describe("IconTile", () => {
  it("tints the tile only with a plain hex color", () => {
    const { container } = render(<IconTile kind="ssh" color="url(javascript:x)" />);

    expect(container.querySelector("[style]")).toBeNull();
  });
});

describe("GroupHeader", () => {
  it("is a heading", () => {
    render(<GroupHeader>Rotta</GroupHeader>);

    expect(screen.getByRole("heading", { name: "Rotta", level: 3 })).toBeInTheDocument();
  });
});

describe("Logo", () => {
  it("is named Portolano without the wordmark", () => {
    render(<Logo wordmark={false} />);

    expect(screen.getByRole("img", { name: "Portolano" })).toBeInTheDocument();
  });

  it("shows the lowercase wordmark", () => {
    render(<Logo />);

    expect(screen.getByText("portolano")).toBeInTheDocument();
  });
});

describe("KeyValueGrid with text of the document", () => {
  it("shows control characters of labels and values", () => {
    render(<KeyValueGrid rows={[{ key: "p", label: "Ambiente\u0007", value: "prod\u001b" }]} />);

    expect(screen.getByRole("term")).toHaveTextContent("Ambiente\\u{7}");
    expect(screen.getByRole("definition")).toHaveTextContent("prod\\u{1b}");
  });
});
