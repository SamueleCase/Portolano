import { describe, expect, it } from "vitest";

import { createPoints, movePoints, seededRandom } from "./constellation";

describe("constellation", () => {
  it("has 120 points with speeds within 13 px per second", () => {
    const points = createPoints(seededRandom(1));

    expect(points).toHaveLength(120);
    expect(points.every((point) => Math.abs(point.vx) <= 13 && Math.abs(point.vy) <= 13)).toBe(
      true,
    );
  });

  it("is the same sky for the same seed", () => {
    expect(createPoints(seededRandom(7))).toEqual(createPoints(seededRandom(7)));
  });

  it("wraps a point that leaves on one side back in from the other", () => {
    const points = [{ x: 0.99, y: 0.5, vx: 20, vy: 0, big: false }];

    movePoints(points, 1, 1000, 800);

    expect(points[0]?.x).toBeCloseTo(0.01, 5);
  });

  it("leaves the points still when the step is zero", () => {
    const points = createPoints(seededRandom(3));
    const before = structuredClone(points);

    movePoints(points, 0, 1280, 800);

    expect(points).toEqual(before);
  });
});
