import { describe, expect, it } from "vitest";

import { interpolate, plural } from "./format";

const nodes = { one: "{n} nodo", other: "{n} nodi" };

describe("plural", () => {
  it("uses the singular for one", () => {
    expect(plural(1, nodes)).toBe("1 nodo");
  });

  it("uses the plural for zero", () => {
    expect(plural(0, nodes)).toBe("0 nodi");
  });

  it("writes large counts with the Italian grouping", () => {
    expect(plural(12000, nodes)).toBe("12.000 nodi");
  });
});

describe("interpolate", () => {
  it("replaces each placeholder with its value", () => {
    expect(interpolate("Scegli {name}", { name: "bastion" })).toBe("Scegli bastion");
  });

  it("leaves a placeholder without a value as it is", () => {
    expect(interpolate("Sposta {name}", {})).toBe("Sposta {name}");
  });

  it("does not expand placeholders inside a value", () => {
    expect(interpolate("{a} e {b}", { a: "{b}", b: "x" })).toBe("{b} e x");
  });
});
