import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { TreeRow } from "./TreeRow";

describe("TreeRow", () => {
  it("is a tree item with its level, selection and expansion", () => {
    render(<TreeRow name="Produzione" kind="folder" depth={0} expanded selected />);

    const row = screen.getByRole("treeitem");
    expect(row).toHaveAttribute("aria-level", "1");
    expect(row).toHaveAttribute("aria-expanded", "true");
    expect(row).toHaveAttribute("aria-selected", "true");
  });

  it("has no expansion state for a connection", () => {
    render(<TreeRow name="web01" kind="ssh" depth={1} />);

    expect(screen.getByRole("treeitem")).not.toHaveAttribute("aria-expanded");
  });

  it("shows the host of a connection", () => {
    render(
      <TreeRow
        name="web01"
        kind="ssh"
        depth={1}
        meta={{ kind: "host", host: "web01.example.com" }}
      />,
    );

    expect(screen.getByText("web01.example.com")).toBeInTheDocument();
  });

  it("counts the descendants of a closed folder (ambiguity 21)", () => {
    render(
      <TreeRow name="Staging" kind="folder" depth={0} meta={{ kind: "descendants", count: 2 }} />,
    );

    expect(screen.getByText("2 elementi")).toBeInTheDocument();
  });

  it("hides the count of an open folder", () => {
    render(
      <TreeRow
        name="Staging"
        kind="folder"
        depth={0}
        expanded
        meta={{ kind: "descendants", count: 2 }}
      />,
    );

    expect(screen.queryByText("2 elementi")).not.toBeInTheDocument();
  });

  it("says vault bloccato for a credential while the vault is locked", () => {
    render(
      <TreeRow
        name="deploy"
        kind="credential"
        depth={1}
        meta={{ kind: "secret", state: "locked", backend: "file" }}
      />,
    );

    expect(screen.getByText("vault bloccato")).toBeInTheDocument();
  });

  it("marks an unsupported node", () => {
    render(<TreeRow name="vnc-legacy" kind="unsupported" depth={0} />);

    expect(screen.getByText("non supportato")).toBeInTheDocument();
  });

  it("names the favorite star and the session dot", () => {
    render(<TreeRow name="bastion" kind="ssh" depth={1} favorite session="connected" />);

    expect(screen.getByRole("img", { name: "Preferito" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Connesso" })).toBeInTheDocument();
  });

  it("shows control characters of the name visibly", () => {
    render(<TreeRow name={"web\u001b[2J01"} kind="ssh" depth={0} />);

    expect(screen.getByRole("treeitem")).toHaveTextContent("web\\u{1b}[2J01");
  });

  it("marks the part of the name that matches the search", () => {
    render(<TreeRow name="bastion" kind="ssh" depth={0} highlight="BAST" />);

    expect(screen.getByText("bast").tagName).toBe("MARK");
  });

  it("draws the color dot only for a plain hex color", () => {
    const { container } = render(
      <TreeRow name="web01" kind="ssh" depth={0} color="red; background: url(x)" />,
    );

    expect(container.querySelector("[style*='background']")).toBeNull();
  });
});
