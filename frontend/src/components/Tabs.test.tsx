import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { Tabs, type WorkTab } from "./Tabs";

const tabs: WorkTab[] = [
  { id: "detail", kind: "detail", title: "web01", dot: "detail", closable: false },
  {
    id: "validation",
    kind: "validation",
    title: "Validazione",
    dot: "validation",
    closable: true,
    problems: 4,
  },
  { id: "s1", kind: "ssh", title: "db02", dot: "failed", closable: true },
];

function WorkTabs({ onClose = vi.fn() }: { onClose?: (id: string) => void }) {
  const [selected, setSelected] = useState("detail");
  return (
    <Tabs
      tabs={tabs}
      selectedId={selected}
      onSelect={setSelected}
      onClose={onClose}
      panelId="work"
    />
  );
}

describe("Tabs", () => {
  it("is a tab list with the selected tab", () => {
    render(<WorkTabs />);

    expect(screen.getByRole("tablist", { name: "Schede di lavoro" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "web01 dettaglio" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("counts the problems in the validation tab", () => {
    render(<WorkTabs />);

    expect(screen.getByRole("tab", { name: "Validazione 4 problemi" })).toBeInTheDocument();
  });

  it("tells the session state in the name of a session tab", () => {
    render(<WorkTabs />);

    expect(
      screen.getByRole("tab", { name: "Connessione non riuscita db02 SSH" }),
    ).toBeInTheDocument();
  });

  it("selects the next tab with the right arrow", async () => {
    render(<WorkTabs />);

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}");

    const validation = screen.getByRole("tab", { name: "Validazione 4 problemi" });
    expect(validation).toHaveFocus();
    expect(validation).toHaveAttribute("aria-selected", "true");
  });

  it("closes the focused tab with Delete", async () => {
    const onClose = vi.fn();
    render(<WorkTabs onClose={onClose} />);

    await userEvent.tab();
    await userEvent.keyboard("{End}{Delete}");

    expect(onClose).toHaveBeenCalledWith("s1");
  });

  it("does not close the detail tab", async () => {
    const onClose = vi.fn();
    render(<WorkTabs onClose={onClose} />);

    await userEvent.tab();
    await userEvent.keyboard("{Delete}");

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "Chiudi web01" })).not.toBeInTheDocument();
  });

  it("closes a tab with its ×", async () => {
    const onClose = vi.fn();
    render(<WorkTabs onClose={onClose} />);

    await userEvent.click(screen.getByRole("button", { name: "Chiudi db02" }));

    expect(onClose).toHaveBeenCalledWith("s1");
  });
});
