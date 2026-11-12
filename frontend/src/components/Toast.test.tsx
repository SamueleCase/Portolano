import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ToastMessage } from "./Toast";
import { ToastViewport } from "./ToastViewport";

const saved: ToastMessage = { id: "1", kind: "success", text: "Password salvata nel portachiavi" };
const refused: ToastMessage = { id: "2", kind: "error", text: "db02: connessione rifiutata" };

describe("ToastViewport", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("is a polite live region named Notifiche", () => {
    render(<ToastViewport toasts={[saved]} onDismiss={vi.fn()} />);

    const region = screen.getByRole("region", { name: "Notifiche" });
    expect(within(region).getByRole("list")).toHaveAttribute("aria-live", "polite");
  });

  it("dismisses a confirmation after 5 seconds", () => {
    const onDismiss = vi.fn();
    render(<ToastViewport toasts={[saved]} onDismiss={onDismiss} />);

    act(() => {
      vi.advanceTimersByTime(4999);
    });
    expect(onDismiss).not.toHaveBeenCalled();
    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(onDismiss).toHaveBeenCalledWith("1");
  });

  it("keeps an undo toast for 10 seconds", () => {
    const onDismiss = vi.fn();
    const undo: ToastMessage = {
      id: "3",
      kind: "undo",
      text: "web02 eliminato",
      action: { label: "Annulla", onAction: vi.fn() },
    };
    render(<ToastViewport toasts={[undo]} onDismiss={onDismiss} />);

    act(() => {
      vi.advanceTimersByTime(9999);
    });
    expect(onDismiss).not.toHaveBeenCalled();
    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(onDismiss).toHaveBeenCalledWith("3");
  });

  it("keeps an error until it is closed with ×", () => {
    const onDismiss = vi.fn();
    render(<ToastViewport toasts={[refused]} onDismiss={onDismiss} />);

    act(() => {
      vi.advanceTimersByTime(60_000);
    });
    expect(onDismiss).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Chiudi la notifica" }));

    expect(onDismiss).toHaveBeenCalledWith("2");
  });

  it("announces an error as an alert", () => {
    render(<ToastViewport toasts={[refused]} onDismiss={vi.fn()} />);

    expect(screen.getByRole("alert")).toHaveTextContent("db02: connessione rifiutata");
  });

  it("waits while the pointer is on the toast", () => {
    const onDismiss = vi.fn();
    render(<ToastViewport toasts={[saved]} onDismiss={onDismiss} />);

    fireEvent.pointerEnter(screen.getByRole("listitem"));
    act(() => {
      vi.advanceTimersByTime(10_000);
    });

    expect(onDismiss).not.toHaveBeenCalled();
  });

  it("runs the action and dismisses the toast", () => {
    const onAction = vi.fn();
    const onDismiss = vi.fn();
    const undo: ToastMessage = {
      id: "4",
      kind: "undo",
      text: "web02 eliminato",
      action: { label: "Annulla", onAction },
    };
    render(<ToastViewport toasts={[undo]} onDismiss={onDismiss} />);

    fireEvent.click(screen.getByRole("button", { name: "Annulla" }));

    expect(onAction).toHaveBeenCalledOnce();
    expect(onDismiss).toHaveBeenCalledWith("4");
  });

  it("shows only the newest three", () => {
    const many = ["a", "b", "c", "d"].map((id) => ({ ...saved, id, text: `Toast ${id}` }));
    render(<ToastViewport toasts={many} onDismiss={vi.fn()} />);

    expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual([
      "Toast b",
      "Toast c",
      "Toast d",
    ]);
  });
});
