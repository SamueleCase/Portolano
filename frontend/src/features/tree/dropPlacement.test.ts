import { describe, expect, it } from "vitest";

import { dropPlacement } from "./dropPlacement";

describe("the place of a dropped node", () => {
  it("goes before the shown node it was dropped above", () => {
    expect(dropPlacement(["a", "b", "c"], 1, "c")).toEqual({ kind: "before", id: "b" });
  });

  it("skips the dragged node itself", () => {
    expect(dropPlacement(["a", "b", "c"], 1, "b")).toEqual({ kind: "before", id: "c" });
  });

  it("goes after the last shown node when dropped at the end", () => {
    expect(dropPlacement(["a", "b"], 2, "x")).toEqual({ kind: "after", id: "b" });
  });

  it("goes last into a folder without other shown children", () => {
    expect(dropPlacement(["x"], 1, "x")).toEqual({ kind: "end" });
  });
});
