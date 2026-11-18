import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { EditorNotices } from "./EditorNotices";
import { GroupError } from "./GroupError";

/** A message of the backend quoting a name with a bidi override and an escape. */
const QUOTING = "opzione 'web‮01\u001b[31m' rifiutata";
const VISIBLE = "opzione 'web\\u{202e}01\\u{1b}[31m' rifiutata";

describe("the errors of the editor", () => {
  it("show the error of a group with its controls visible", () => {
    render(<GroupError id="errore" text={QUOTING} />);

    expect(screen.getByRole("alert")).toHaveTextContent(VISIBLE);
  });

  it("show the error without a field with its controls visible", () => {
    render(<EditorNotices external={undefined} error={QUOTING} secretsLost={false} />);

    expect(screen.getByRole("alert")).toHaveTextContent(VISIBLE);
  });
});
