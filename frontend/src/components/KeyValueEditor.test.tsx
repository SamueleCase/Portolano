import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { type KeyValueEntry, KeyValueEditor } from "./KeyValueEditor";

const properties: KeyValueEntry[] = [
  { id: "1", key: "Ambiente", type: "text", value: "produzione" },
  { id: "2", key: "Token API", type: "secret", value: "", secret: "present" },
];

function renderEditor(overrides: Partial<Parameters<typeof KeyValueEditor>[0]> = {}) {
  const props = {
    label: "Proprietà custom",
    variant: "properties" as const,
    entries: properties,
    backend: "keychain" as const,
    addLabel: "+ Proprietà",
    onChange: vi.fn(),
    onReplaceSecret: vi.fn(),
    onRemove: vi.fn(),
    onAdd: vi.fn(),
    ...overrides,
  };
  render(<KeyValueEditor {...props} />);
  return props;
}

describe("KeyValueEditor", () => {
  it("shows the state of a secret property, never a value", () => {
    renderEditor();

    expect(screen.getByText("Nel portachiavi")).toBeInTheDocument();
    expect(screen.getAllByRole("textbox")).toHaveLength(3);
  });

  it("replaces a secret through the password dialog", async () => {
    const { onReplaceSecret } = renderEditor();

    await userEvent.click(screen.getByRole("button", { name: "Sostituisci" }));

    expect(onReplaceSecret).toHaveBeenCalledWith("2");
  });

  it("reports an edited value", async () => {
    const { onChange } = renderEditor();

    await userEvent.type(screen.getByDisplayValue("produzione"), "!");

    expect(onChange).toHaveBeenCalledWith("1", { value: "produzione!" });
  });

  it("removes an entry by its name", async () => {
    const { onRemove } = renderEditor();

    await userEvent.click(screen.getByRole("button", { name: "Rimuovi Token API" }));

    expect(onRemove).toHaveBeenCalledWith("2");
  });

  it("shows the message of the backend under a refused option", () => {
    renderEditor({
      variant: "options",
      entries: [
        {
          id: "o",
          key: "ProxyCommand",
          type: "text",
          value: "nc %h %p",
          error: "Opzione non ammessa: aggiungila al tuo ~/.ssh/config",
        },
      ],
    });

    expect(screen.getByDisplayValue("ProxyCommand")).toHaveAttribute("aria-invalid", "true");
    expect(
      screen.getByText("Opzione non ammessa: aggiungila al tuo ~/.ssh/config"),
    ).toBeInTheDocument();
  });
});
