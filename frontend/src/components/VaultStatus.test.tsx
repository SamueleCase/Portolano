import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { VaultStatus } from "./VaultStatus";

describe("VaultStatus", () => {
  it("names the system keychain without a state or an action (ambiguity 20)", () => {
    render(<VaultStatus vault={{ backend: "keychain" }} nodeCount={12} onUnlock={vi.fn()} />);

    expect(screen.getByText("Portachiavi di sistema")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.getByText("12 nodi")).toBeInTheDocument();
  });

  it("opens the unlock dialog from a locked file vault", async () => {
    const onUnlock = vi.fn();
    render(
      <VaultStatus
        vault={{ backend: "file", locked: true, fileName: "infrastruttura.vault" }}
        nodeCount={1}
        onUnlock={onUnlock}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Vault su file · bloccato" }));

    expect(onUnlock).toHaveBeenCalledOnce();
    expect(screen.getByText("1 nodo")).toBeInTheDocument();
  });

  it("shows an unlocked file vault without an unlock action", () => {
    render(
      <VaultStatus
        vault={{ backend: "file", locked: false, fileName: "infrastruttura.vault" }}
        nodeCount={3}
        onUnlock={vi.fn()}
      />,
    );

    expect(screen.getByText("Vault su file · sbloccato")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
