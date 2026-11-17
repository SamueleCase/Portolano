import { describe, expect, it } from "vitest";

import { readableNames } from "./readablePath";

describe("readableNames", () => {
  it("writes names from the root as a path of the interface, slashes inside them included", () => {
    expect(readableNames(["Clienti", "ACME/Nord", "web01"])).toBe("Clienti / ACME/Nord / web01");
  });

  it("writes a node at the root as its name alone", () => {
    expect(readableNames(["web01"])).toBe("web01");
  });
});
