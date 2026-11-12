import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ReportRow } from "./ReportRow";
import { ValidationRow } from "./ValidationRow";

describe("ValidationRow", () => {
  it("leads to the node, errors included", async () => {
    const onGoToNode = vi.fn();
    render(
      <ul>
        <ValidationRow
          severity="error"
          path="Archivio / old-gw"
          message="Il jump host fa riferimento a un nodo che non esiste più."
          onGoToNode={onGoToNode}
        />
      </ul>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Vai al nodo →" }));

    expect(screen.getByText("Errore")).toBeInTheDocument();
    expect(onGoToNode).toHaveBeenCalledOnce();
  });

  it("shows control characters of the path visibly", () => {
    render(
      <ul>
        <ValidationRow
          severity="info"
          path={"vnc\u001b"}
          message="Nodo non supportato."
          onGoToNode={vi.fn()}
        />
      </ul>,
    );

    expect(screen.getByText("vnc\\u{1b}")).toBeInTheDocument();
  });
});

describe("ReportRow", () => {
  it("leads to the imported node when it exists", async () => {
    const onGoToNode = vi.fn();
    render(
      <ul>
        <ReportRow
          outcome="partial"
          outcomeLabel="Parziale"
          path="Produzione / win-app01"
          reason="Reindirizzamento delle stampanti non gestito, ignorato."
          onGoToNode={onGoToNode}
        />
      </ul>,
    );

    await userEvent.click(screen.getByRole("button"));

    expect(onGoToNode).toHaveBeenCalledOnce();
  });

  it("is plain text for an item of the trash", () => {
    render(
      <ul>
        <ReportRow
          outcome="skipped"
          outcomeLabel="Saltato"
          path="Cestino / vecchio"
          reason="Nel cestino."
        />
      </ul>,
    );

    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
