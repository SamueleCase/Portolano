import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Button } from "./Button";
import { Tooltip } from "./Tooltip";

describe("Tooltip", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  function renderTooltip() {
    render(
      <Tooltip content="Connetti" shortcut="⌘↩">
        <Button>Connetti</Button>
      </Tooltip>,
    );
    return screen.getByRole("button", { name: "Connetti" });
  }

  it("describes its trigger", () => {
    const trigger = renderTooltip();

    expect(trigger).toHaveAccessibleDescription("Connetti⌘↩");
  });

  it("appears 500 ms after the focus arrives", () => {
    const trigger = renderTooltip();

    fireEvent.focus(trigger);
    act(() => {
      vi.advanceTimersByTime(499);
    });
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(screen.getByRole("tooltip")).toBeVisible();
  });

  it("hides on Escape", () => {
    const trigger = renderTooltip();
    fireEvent.focus(trigger);
    act(() => {
      vi.advanceTimersByTime(500);
    });

    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("does not appear when the pointer leaves before the delay", () => {
    const trigger = renderTooltip();

    fireEvent.pointerEnter(trigger);
    act(() => {
      vi.advanceTimersByTime(200);
    });
    fireEvent.pointerLeave(trigger);
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("shows control characters of its text", () => {
    render(
      <Tooltip content={"vault\u0007.vault"}>
        <Button>Vault</Button>
      </Tooltip>,
    );

    expect(screen.getByRole("button", { name: "Vault" })).toHaveAccessibleDescription(
      "vault\\u{7}.vault",
    );
  });

  describe("in the window", () => {
    /** A tooltip 120 × 30 px for a trigger 40 × 20 px at `left`, `top`. */
    function boxes(left: number, top: number) {
      vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
        this: HTMLElement,
      ) {
        return this.getAttribute("role") === "tooltip"
          ? new DOMRect(0, 0, 120, 30)
          : new DOMRect(left, top, 40, 20);
      });
    }

    function shown() {
      fireEvent.focus(renderTooltip());
      act(() => {
        vi.advanceTimersByTime(500);
      });
      return screen.getByRole("tooltip");
    }

    it("opens under its trigger, centered, when there is room", () => {
      boxes(400, 100);

      const tooltip = shown();

      expect(tooltip.style.top).toBe("120px");
      expect(tooltip.style.left).toBe("360px");
    });

    it("opens above its trigger when the window has no room below", () => {
      boxes(400, window.innerHeight - 30);

      const tooltip = shown();

      expect(tooltip.style.top).toBe("");
      expect(tooltip.style.bottom).toBe("30px");
    });

    it("stays inside the window by its right edge", () => {
      boxes(window.innerWidth - 50, 100);

      expect(shown().style.left).toBe(`${window.innerWidth - 120}px`);
    });

    it("stays inside the window by its left edge", () => {
      boxes(0, 100);

      expect(shown().style.left).toBe("0px");
    });
  });
});
