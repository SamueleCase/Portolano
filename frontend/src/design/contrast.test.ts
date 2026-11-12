import { describe, expect, it } from "vitest";

import { composite, contrastRatio, parseColor } from "./contrast";
import { contrastMinimum, contrastPairs } from "./contrastPairs";
import { type TokenName, tokens } from "./tokens";

function ratioOf(foreground: readonly TokenName[], surface: readonly TokenName[]): number {
  const values = (names: readonly TokenName[]) => names.map((name) => tokens[name]);
  const background = composite(values(surface));
  const front = composite([...values(surface), ...values(foreground)]);
  return contrastRatio(front, background);
}

describe("contrast", () => {
  it("computes the WCAG ratio of black on white", () => {
    expect(contrastRatio(parseColor("#000000"), parseColor("#FFFFFF"))).toBeCloseTo(21, 5);
  });

  it("blends translucent layers over the opaque bottom", () => {
    expect(composite(["#000000", "rgba(255,255,255,.5)"])).toEqual({
      red: 127.5,
      green: 127.5,
      blue: 127.5,
      alpha: 1,
    });
  });

  it("reproduces the ratio of the panel text measured in the inventory", () => {
    // inventario §7: `--tx` on a panel is 13.68:1.
    expect(ratioOf(["--color-text-primary"], ["--color-bg", "--color-glass-panel"])).toBeCloseTo(
      13.68,
      1,
    );
  });

  it.each(contrastPairs.map((pair) => [pair.usage, pair] as const))(
    "%s reaches the WCAG minimum",
    (_usage, pair) => {
      expect(ratioOf(pair.foreground, pair.surface)).toBeGreaterThanOrEqual(
        contrastMinimum[pair.kind],
      );
    },
  );
});
