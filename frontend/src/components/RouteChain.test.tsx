import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { RouteChain } from "./RouteChain";
import { SessionRoute } from "./SessionRoute";
import { Stepper } from "./Stepper";

describe("RouteChain", () => {
  it("starts the route from tu", () => {
    render(
      <RouteChain
        variant="route"
        stops={[{ label: "bastion.example.com" }, { label: "web01.example.com", kind: "ssh" }]}
      />,
    );

    const route = screen.getByRole("list", { name: "Rotta" });
    expect(
      within(route)
        .getAllByRole("listitem")
        .map((item) => item.textContent)
        .filter(Boolean),
    ).toEqual(["tu", "bastion.example.com", "web01.example.com"]);
  });

  it("shows the chain of an inherited credential without tu", () => {
    render(
      <RouteChain
        variant="inherit"
        label="Da dove arriva"
        stops={[
          { label: "web01" },
          { label: "cartella Produzione" },
          { label: "credenziale deploy" },
        ]}
      />,
    );

    expect(screen.queryByText("tu")).not.toBeInTheDocument();
  });

  it("shows control characters of a host visibly", () => {
    render(<RouteChain variant="editor" stops={[{ label: "gw\u0007" }]} />);

    expect(screen.getByText("gw\\u{7}")).toBeInTheDocument();
  });
});

describe("SessionRoute", () => {
  it("tells the phase of the connection in words", () => {
    render(<SessionRoute stops={["bastion", "db02"]} phase="failed" />);

    expect(screen.getByText("Connessione non riuscita")).toBeInTheDocument();
  });
});

describe("Stepper", () => {
  const steps = [
    { label: "File", state: "done" },
    { label: "Opzioni", state: "current" },
    { label: "Password", state: "skipped" },
    { label: "Report", state: "upcoming" },
  ] as const;

  it("marks the current step", () => {
    render(<Stepper steps={steps} />);

    expect(screen.getByText(/Opzioni/).closest("li")).toHaveAttribute("aria-current", "step");
  });

  it("says that a step is skipped", () => {
    render(<Stepper steps={steps} />);

    expect(screen.getByText("(saltato)", { exact: false })).toBeInTheDocument();
  });

  it("makes the steps of the editor buttons", () => {
    render(<Stepper steps={steps} onSelect={() => undefined} variant="editor" />);

    expect(screen.getAllByRole("button")).toHaveLength(4);
  });
});
