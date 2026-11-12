import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { backgroundControl } from "../design/background/backgroundControl";
import { Button } from "./Button";
import { Dialog } from "./Dialog";
import { TextField } from "./TextField";

function PasswordDialog({ onClose = vi.fn() }: { onClose?: () => void }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button onClick={() => setOpen(true)}>Imposta…</Button>
      <Dialog
        open={open}
        onClose={() => {
          setOpen(false);
          onClose();
        }}
        title="Imposta la password"
        subtitle="Credenziali / deploy"
        footer={
          <>
            <Button onClick={() => setOpen(false)}>Annulla</Button>
            <Button variant="primary">Imposta</Button>
          </>
        }
      >
        <TextField label="Password" type="password" />
      </Dialog>
    </>
  );
}

async function openDialog() {
  await userEvent.click(screen.getByRole("button", { name: "Imposta…" }));
  return screen.getByRole("dialog", { name: "Imposta la password" });
}

describe("Dialog", () => {
  it("is a modal dialog named by its title", async () => {
    render(<PasswordDialog />);

    const dialog = await openDialog();

    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(screen.getByText("Credenziali / deploy")).toBeInTheDocument();
  });

  it("moves the focus to its first field", async () => {
    render(<PasswordDialog />);

    await openDialog();

    expect(screen.getByLabelText("Password")).toHaveFocus();
  });

  it("keeps the focus inside with Tab", async () => {
    render(<PasswordDialog />);
    await openDialog();

    await userEvent.tab();
    await userEvent.tab();
    await userEvent.tab();

    expect(screen.getByLabelText("Password")).toHaveFocus();
  });

  it("keeps the focus inside with Shift+Tab", async () => {
    render(<PasswordDialog />);
    await openDialog();

    await userEvent.tab({ shift: true });

    expect(screen.getByRole("button", { name: "Imposta" })).toHaveFocus();
  });

  it("closes on Escape and gives the focus back", async () => {
    const onClose = vi.fn();
    render(<PasswordDialog onClose={onClose} />);
    await openDialog();

    await userEvent.keyboard("{Escape}");

    expect(onClose).toHaveBeenCalledOnce();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Imposta…" })).toHaveFocus();
  });

  it("closes on a click on the veil", async () => {
    const onClose = vi.fn();
    render(<PasswordDialog onClose={onClose} />);
    const dialog = await openDialog();

    await userEvent.click(dialog.parentElement ?? dialog);

    expect(onClose).toHaveBeenCalledOnce();
  });

  it("stays open on a click inside", async () => {
    const onClose = vi.fn();
    render(<PasswordDialog onClose={onClose} />);
    const dialog = await openDialog();

    await userEvent.click(dialog);

    expect(onClose).not.toHaveBeenCalled();
  });

  it("slows the background down while open (ambiguity 4)", async () => {
    render(<PasswordDialog />);

    await openDialog();
    const whileOpen = backgroundControl.inputs().openDialogs;
    await userEvent.keyboard("{Escape}");

    expect(whileOpen).toBe(1);
    expect(backgroundControl.inputs().openDialogs).toBe(0);
  });

  it("can be named without a visible title", () => {
    render(<Dialog open onClose={vi.fn()} label="Credenziale" />);

    expect(screen.getByRole("dialog", { name: "Credenziale" })).toBeInTheDocument();
  });

  it("shows control characters of a title and a subtitle from the document", () => {
    render(
      <Dialog
        open
        onClose={vi.fn()}
        title={"Elimina web\u001b01"}
        subtitle={"Prod\u0007 / web01"}
      />,
    );

    expect(screen.getByRole("dialog", { name: "Elimina web\\u{1b}01" })).toBeInTheDocument();
    expect(screen.getByText("Prod\\u{7} / web01")).toBeInTheDocument();
  });

  it("runs the default action with Enter in a field", async () => {
    const onSubmit = vi.fn();
    render(
      <Dialog
        open
        onClose={vi.fn()}
        title="Imposta la password"
        onSubmit={onSubmit}
        footer={
          <Button variant="primary" type="submit">
            Imposta
          </Button>
        }
      >
        <TextField label="Password" type="password" />
      </Dialog>,
    );

    await userEvent.keyboard("abc{Enter}");

    expect(onSubmit).toHaveBeenCalledOnce();
  });

  it("can make Annulla the default action, as the delete confirmation does", async () => {
    const onCancel = vi.fn();
    const onDelete = vi.fn();
    render(
      <Dialog
        open
        onClose={onCancel}
        title="Eliminare web01?"
        onSubmit={onCancel}
        footer={
          <>
            <Button type="submit">Annulla</Button>
            <Button variant="danger" onClick={onDelete}>
              Elimina comunque
            </Button>
          </>
        }
      />,
    );

    await userEvent.keyboard("{Enter}");

    expect(screen.getByRole("button", { name: "Annulla" })).toHaveFocus();
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onDelete).not.toHaveBeenCalled();
  });

  it("does not run the default action while its button is disabled", async () => {
    const onSubmit = vi.fn();
    render(
      <Dialog
        open
        onClose={vi.fn()}
        title="Imposta la password"
        onSubmit={onSubmit}
        footer={
          <Button variant="primary" type="submit" disabled>
            Imposta
          </Button>
        }
      >
        <TextField label="Password" type="password" />
      </Dialog>,
    );

    await userEvent.keyboard("{Enter}");

    expect(onSubmit).not.toHaveBeenCalled();
  });
});
