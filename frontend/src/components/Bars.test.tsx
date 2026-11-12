import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { MiniBar } from "./MiniBar";
import { ProgressBar } from "./ProgressBar";
import { StackedBar } from "./StackedBar";
import { StatTile } from "./StatTile";

describe("ProgressBar", () => {
  it("is a progress bar with its value in words", () => {
    render(<ProgressBar label="Password" value={3} max={12} valueText="3 / 12" />);

    const bar = screen.getByRole("progressbar", { name: "Password" });
    expect(bar).toHaveAttribute("aria-valuenow", "3");
    expect(bar).toHaveAttribute("aria-valuetext", "3 / 12");
  });
});

describe("StackedBar", () => {
  it("describes itself and draws no empty segment", () => {
    const { container } = render(
      <StackedBar
        label="118 importati, 9 parziali"
        segments={[
          { outcome: "imported", label: "Importati", count: 118 },
          { outcome: "partial", label: "Parziali", count: 9 },
          { outcome: "skipped", label: "Saltati", count: 0 },
        ]}
      />,
    );

    expect(screen.getByRole("img", { name: "118 importati, 9 parziali" })).toBeInTheDocument();
    expect(container.querySelectorAll("[style]")).toHaveLength(2);
  });
});

describe("StatTile", () => {
  it("shows the count and its share with a decimal comma", () => {
    render(<StatTile outcome="imported" label="Importati" count={118} total={142} />);

    expect(screen.getByText("118")).toBeInTheDocument();
    expect(screen.getByText("83,1%")).toBeInTheDocument();
  });
});

describe("MiniBar", () => {
  it("shows its label and value", () => {
    render(<MiniBar label="SSH" value={68} max={68} kind="ssh" />);

    expect(screen.getByText("SSH")).toBeInTheDocument();
    expect(screen.getByText("68")).toBeInTheDocument();
  });
});
