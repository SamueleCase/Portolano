import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { CountBadge } from "./CountBadge";
import { SeverityMarker } from "./SeverityMarker";
import { StatusBadge } from "./StatusBadge";
import { StatusDot } from "./StatusDot";

describe("StatusBadge", () => {
  it("says the status in words", () => {
    render(<StatusBadge status="connecting" />);

    expect(screen.getByText("In corso")).toBeInTheDocument();
  });
});

describe("CountBadge", () => {
  it("uses the singular for one problem", () => {
    render(<CountBadge severity="error" count={1} />);

    expect(screen.getByText("1 errore")).toBeInTheDocument();
  });

  it("uses the plural for more problems", () => {
    render(<CountBadge severity="info" count={3} />);

    expect(screen.getByText("3 note")).toBeInTheDocument();
  });
});

describe("SeverityMarker", () => {
  it("gives the bare shape the name of its level", () => {
    render(<SeverityMarker severity="warning" />);

    expect(screen.getByRole("img", { name: "Avviso" })).toBeInTheDocument();
  });

  it("shows the word next to the shape", () => {
    render(<SeverityMarker severity="error" withLabel />);

    expect(screen.getByText("Errore")).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});

describe("StatusDot", () => {
  it("tells its state as text", () => {
    render(<StatusDot state="failed" />);

    expect(screen.getByRole("img", { name: "Connessione non riuscita" })).toBeInTheDocument();
  });

  it("is silent when there is no session", () => {
    render(<StatusDot state="idle" />);

    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});
