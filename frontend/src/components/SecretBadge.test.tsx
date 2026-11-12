import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SecretBadge } from "./SecretBadge";

describe("SecretBadge", () => {
  it("names the keychain for a present secret with the keychain backend", () => {
    render(<SecretBadge state="present" backend="keychain" />);

    expect(screen.getByText("Nel portachiavi")).toBeInTheDocument();
  });

  it("names the vault for a present secret with the file backend", () => {
    render(<SecretBadge state="present" backend="file" />);

    expect(screen.getByText("Nel vault")).toBeInTheDocument();
  });

  it("says the vault is locked instead of absent", () => {
    render(<SecretBadge state="locked" backend="file" />);

    expect(screen.getByText("Vault bloccato")).toBeInTheDocument();
    expect(screen.queryByText("Assente")).not.toBeInTheDocument();
  });

  it("marks a staged secret as to be saved (ambiguity 14)", () => {
    render(<SecretBadge state="staged" backend="keychain" />);

    expect(screen.getByText("Da salvare")).toBeInTheDocument();
  });

  it("says password assente in the compact variant", () => {
    render(<SecretBadge state="absent" backend="keychain" variant="compact" />);

    expect(screen.getByText("password assente")).toBeInTheDocument();
  });
});
