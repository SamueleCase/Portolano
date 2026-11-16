import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { texts } from "../../i18n/it";
import { SecretDialog } from "./SecretDialog";

const t = texts.dialogs.secret;
const SENTINEL = "sentinel-dialog-secret-5d1e";

function renderDialog(slot: Parameters<typeof SecretDialog>[0]["slot"] = { kind: "password" }) {
  const onConfirm = vi.fn();
  render(
    <SecretDialog
      open
      slot={slot}
      nodeLabel="Clienti / Admin ACME"
      replacing={false}
      mode="stage"
      backend="file"
      onConfirm={onConfirm}
      onClose={() => undefined}
    />,
  );
  return onConfirm;
}

describe("SecretDialog", () => {
  it("names the field and the node in its title", () => {
    renderDialog({ kind: "property", name: "Token API" });

    expect(screen.getByRole("dialog", { name: "Imposta «Token API»" })).toBeInTheDocument();
    expect(screen.getByText("Clienti / Admin ACME")).toBeInTheDocument();
  });

  it("hands over the value once and empties its fields", async () => {
    const onConfirm = renderDialog();
    await userEvent.type(screen.getByLabelText(t.fields.password), SENTINEL);
    await userEvent.type(screen.getByLabelText(t.confirm), SENTINEL);

    await userEvent.click(screen.getByRole("button", { name: t.stage }));

    expect(onConfirm).toHaveBeenCalledExactlyOnceWith(SENTINEL);
    expect(screen.getByLabelText(t.fields.password)).toHaveValue("");
    expect(screen.getByLabelText(t.confirm)).toHaveValue("");
    expect(document.body.innerHTML).not.toContain(SENTINEL);
  });

  it("says when the two values match", async () => {
    renderDialog();
    await userEvent.type(screen.getByLabelText(t.fields.password), "uguale");
    await userEvent.type(screen.getByLabelText(t.confirm), "uguale");

    expect(screen.getByText(t.match.password)).toBeInTheDocument();
  });

  it("refuses two different values", async () => {
    const onConfirm = renderDialog();
    await userEvent.type(screen.getByLabelText(t.fields.password), "una");
    await userEvent.type(screen.getByLabelText(t.confirm), "altra");

    await userEvent.click(screen.getByRole("button", { name: t.stage }));

    expect(onConfirm).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(t.mismatch.password);
  });

  it("refuses an empty value", async () => {
    const onConfirm = renderDialog();

    await userEvent.click(screen.getByRole("button", { name: t.stage }));

    expect(onConfirm).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(t.empty);
  });

  it("takes a private key with its line breaks in a single masked area", async () => {
    const onConfirm = renderDialog({ kind: "privateKey" });

    await userEvent.type(screen.getByLabelText(t.fields.privateKey), "riga 1{Enter}riga 2");
    await userEvent.click(screen.getByRole("button", { name: t.stage }));

    expect(screen.queryByLabelText(t.confirm)).not.toBeInTheDocument();
    expect(onConfirm).toHaveBeenCalledExactlyOnceWith("riga 1\nriga 2");
  });

  it("lets nothing be copied out of the private key", async () => {
    renderDialog({ kind: "privateKey" });
    const area = screen.getByLabelText(t.fields.privateKey);
    await userEvent.type(area, "chiave");

    const copy = new Event("copy", { bubbles: true, cancelable: true });
    area.dispatchEvent(copy);

    expect(copy.defaultPrevented).toBe(true);
  });

  it("puts new fields in place of those whose value was sent", async () => {
    renderDialog();
    const first = screen.getByLabelText(t.fields.password);
    await userEvent.type(first, "uno");
    await userEvent.type(screen.getByLabelText(t.confirm), "uno");

    await userEvent.click(screen.getByRole("button", { name: t.stage }));

    expect(screen.getByLabelText(t.fields.password)).not.toBe(first);
  });
});
