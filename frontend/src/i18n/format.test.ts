import { describe, expect, it } from "vitest";

import { interpolate, percent, plural } from "./format";

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

describe("percent", () => {
  it("writes a share with a decimal comma", () => {
    expect(percent(118, 142)).toBe("83,1%");
  });

  it("is zero for an empty total", () => {
    expect(percent(0, 0)).toBe("0,0%");
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
