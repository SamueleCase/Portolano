import { describe, expect, it } from "vitest";

import { typeaheadMatch } from "./typeahead";

const NAMES = ["Clienti", "ACME", "Admin ACME", "db01", "deploy"];

describe("the typeahead of the tree", () => {
  it("finds the next name that starts with the letters, ignoring case", () => {
    expect(typeaheadMatch(NAMES, 0, "de")).toBe(4);
  });

  it("moves on to the next name with the same letter when one letter is typed", () => {
    expect(typeaheadMatch(NAMES, 1, "a")).toBe(2);
  });

  it("goes round to the first names", () => {
    expect(typeaheadMatch(NAMES, 4, "c")).toBe(0);
  });

  it("finds nothing for letters no name starts with", () => {
    expect(typeaheadMatch(NAMES, 0, "zz")).toBeUndefined();
  });
});
