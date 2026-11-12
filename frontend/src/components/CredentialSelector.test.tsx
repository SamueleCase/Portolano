import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { CredentialPreview } from "./CredentialPreview";
import type { CredentialPreviewData } from "./credentialPreviewData";
import { CredentialSelector } from "./CredentialSelector";

function renderSelector(onModeChange = vi.fn(), onDone = vi.fn()) {
  render(
    <CredentialSelector
      open
      onClose={vi.fn()}
      onDone={onDone}
      mode="none"
      onModeChange={onModeChange}
      preview={{ mode: "none" }}
      backend="keychain"
    />,
  );
  return { onModeChange, onDone };
}

describe("CredentialSelector", () => {
  it("is a modal dialog with the six modes as a radio group (ambiguity 6)", () => {
    renderSelector();

    expect(screen.getByRole("dialog", { name: "Credenziale" })).toHaveAttribute(
      "aria-modal",
      "true",
    );
    expect(screen.getAllByRole("radio")).toHaveLength(6);
  });

  it("starts with the focus on the chosen mode", () => {
    renderSelector();

    expect(screen.getByRole("radio", { name: /^Nessuna/ })).toHaveFocus();
  });

  it("chooses the next mode with the arrows", async () => {
    const { onModeChange } = renderSelector();

    await userEvent.keyboard("{ArrowDown}");

    expect(onModeChange).toHaveBeenCalledWith("ask");
  });

  it("confirms with Enter, its default action", async () => {
    const { onDone } = renderSelector();

    await userEvent.keyboard("{Enter}");

    expect(onDone).toHaveBeenCalledOnce();
  });

  it("confirms with Fatto", async () => {
    const { onDone } = renderSelector();

    await userEvent.click(screen.getByRole("button", { name: "Fatto" }));

    expect(onDone).toHaveBeenCalledOnce();
  });
});

function preview(data: CredentialPreviewData) {
  render(<CredentialPreview data={data} backend="keychain" />);
}

describe("CredentialPreview", () => {
  it("draws the chain of an inherited credential", () => {
    preview({
      mode: "inherit",
      node: "web01",
      source: {
        chain: ["web01", "cartella Produzione", "credenziale deploy"],
        credential: { user: "deploy", password: "present", key: "Nel portachiavi, con passphrase" },
      },
    });

    expect(screen.getByText("credenziale deploy")).toBeInTheDocument();
    expect(screen.getByText("Nel portachiavi")).toBeInTheDocument();
  });

  it("says when no folder has a credential to inherit (ambiguity 23)", () => {
    preview({ mode: "inherit", node: "web01", source: null });

    expect(
      screen.getByRole("heading", { name: "Nessuna credenziale ereditata" }),
    ).toBeInTheDocument();
  });

  it("says when no credential has the name", () => {
    preview({ mode: "byName", name: "deploy", match: { kind: "notFound" } });

    expect(screen.getByRole("status")).toHaveTextContent("Nessuna credenziale ha questo nome");
  });

  it("lists the credentials that share an ambiguous name", () => {
    preview({
      mode: "byName",
      name: "admin",
      match: { kind: "ambiguous", paths: ["Credenziali / admin", "Staging / admin"] },
    });

    expect(screen.getByRole("status")).toHaveTextContent("Più credenziali hanno questo nome");
    expect(screen.getByText("Staging / admin")).toBeInTheDocument();
  });

  it("shows the match of a name found in the document", () => {
    preview({
      mode: "byName",
      name: "deploy",
      match: { kind: "found", path: "Credenziali / deploy", resolved: { user: "deploy" } },
    });

    expect(screen.getByText("Credenziali / deploy")).toBeInTheDocument();
  });

  it("offers no proposed user when asking at launch (DEC-016.3)", () => {
    preview({ mode: "ask" });

    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("edits the user of an inline credential", async () => {
    const onInlineChange = vi.fn();
    render(
      <CredentialPreview
        data={{ mode: "inline", user: "", domain: "", password: "absent" }}
        backend="file"
        actions={{ onInlineChange }}
      />,
    );

    await userEvent.type(screen.getByRole("textbox", { name: "Utente" }), "a");

    expect(onInlineChange).toHaveBeenCalledWith("user", "a");
  });
});
