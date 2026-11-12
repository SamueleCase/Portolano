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
});
