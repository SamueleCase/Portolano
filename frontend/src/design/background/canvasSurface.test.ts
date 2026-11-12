import { describe, expect, it } from "vitest";

import { bitmapScale } from "./canvasSurface";

describe("bitmapScale", () => {
  it("follows the device pixel ratio for the window", () => {
    expect(bitmapScale(2, 1280, 800)).toBe(2);
  });

  it("lowers the resolution of a very tall canvas", () => {
    const scale = bitmapScale(2, 1280, 14_000);

    expect(1280 * 14_000 * scale * scale).toBeLessThanOrEqual(4096 * 4096 + 1);
  });

  it("treats a missing ratio as 1", () => {
    expect(bitmapScale(0, 800, 600)).toBe(1);
  });
});
