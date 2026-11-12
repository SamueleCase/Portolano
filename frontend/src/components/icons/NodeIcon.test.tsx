import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { NodeIcon } from "./NodeIcon";

const hash = "0123456789abcdef".repeat(4);

describe("NodeIcon", () => {
  it("shows a custom icon only as an image from the icon scheme", () => {
    const { container } = render(
      <NodeIcon kind="ssh" icon={{ source: "custom", fileName: `${hash}.svg` }} />,
    );

    expect(container.querySelector("img")).toHaveAttribute(
      "src",
      `portolano-icon://localhost/${hash}.svg`,
    );
  });

  it("falls back to the type icon for a custom name that is not a hash", () => {
    const { container } = render(
      <NodeIcon kind="ssh" icon={{ source: "custom", fileName: "javascript:alert(1)" }} />,
    );

    expect(container.querySelector("img")).toBeNull();
    expect(container.querySelector("path")).toHaveAttribute("d", "M3 5l3 3-3 3M8 11h5");
  });

  it("falls back to the type icon for a builtin name outside the set", () => {
    const { container } = render(
      <NodeIcon kind="folder" icon={{ source: "builtin", name: "skull" }} />,
    );

    expect(container.querySelector("path")).toHaveAttribute("d", "M2 4.5h4l1.5 1.5H14v6.5H2z");
  });

  it("draws a builtin icon of the set", () => {
    const { container } = render(
      <NodeIcon kind="rdp" icon={{ source: "builtin", name: "database" }} />,
    );

    expect(container.querySelector("svg.lucide-database")).not.toBeNull();
  });
});
