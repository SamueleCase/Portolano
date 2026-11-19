import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LinkHintBoundary } from "./LinkHintBoundary";

/** A hint that throws while it is drawn. */
function FailingHint(): never {
  throw new Error("riquadro guasto");
}

describe("LinkHintBoundary", () => {
  it("drops a failing hint and keeps what is around it", () => {
    // React reports the error caught by the boundary on the console.
    const reported = vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(
      <div role="group" aria-label="Terminale">
        <LinkHintBoundary>
          <FailingHint />
        </LinkHintBoundary>
      </div>,
    );

    expect(screen.getByRole("group", { name: "Terminale" })).toBeEmptyDOMElement();
    reported.mockRestore();
  });

  it("shows a hint that draws", () => {
    render(
      <LinkHintBoundary>
        <span role="tooltip">www.example.com</span>
      </LinkHintBoundary>,
    );

    expect(screen.getByRole("tooltip")).toHaveTextContent("www.example.com");
  });
});
