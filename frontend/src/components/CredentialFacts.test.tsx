import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { CredentialFacts } from "./CredentialFacts";

describe("CredentialFacts", () => {
  it("shows the key of a credential with its controls visible", () => {
    render(
      <CredentialFacts
        credential={{ user: "deploy", key: "~/.ssh/id‮ed25519\u0007" }}
        backend="file"
      />,
    );

    expect(screen.getByText("~/.ssh/id\\u{202e}ed25519\\u{7}")).toBeInTheDocument();
  });
});
