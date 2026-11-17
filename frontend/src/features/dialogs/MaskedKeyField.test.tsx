import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { MaskedKeyField } from "./MaskedKeyField";

describe("MaskedKeyField", () => {
  it("keeps the line breaks of the key", async () => {
    render(<MaskedKeyField label="Chiave privata" />);
    const area = screen.getByLabelText("Chiave privata");

    await userEvent.type(area, "riga 1{Enter}riga 2");

    expect(area).toHaveValue("riga 1\nriga 2");
  });

  it("offers no autocomplete, correction or spelling on the key", () => {
    render(<MaskedKeyField label="Chiave privata" />);
    const area = screen.getByLabelText("Chiave privata");

    expect(area).toHaveAttribute("autocomplete", "off");
    expect(area).toHaveAttribute("autocorrect", "off");
    expect(area).toHaveAttribute("autocapitalize", "off");
    expect(area).toHaveAttribute("spellcheck", "false");
  });

  it("lets nothing be cut out of the key", () => {
    render(<MaskedKeyField label="Chiave privata" />);

    const cut = new Event("cut", { bubbles: true, cancelable: true });
    screen.getByLabelText("Chiave privata").dispatchEvent(cut);

    expect(cut.defaultPrevented).toBe(true);
  });
});
