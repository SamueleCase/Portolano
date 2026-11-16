import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useDebounced } from "./useDebounced";

describe("useDebounced", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("gives the new value once it stops changing for the delay", () => {
    const { result, rerender } = renderHook(({ value }) => useDebounced(value, 150), {
      initialProps: { value: "w" },
    });

    rerender({ value: "web" });
    act(() => {
      vi.advanceTimersByTime(149);
    });
    expect(result.current).toBe("w");
    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(result.current).toBe("web");
  });
});
