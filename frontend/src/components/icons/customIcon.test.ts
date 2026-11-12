import { describe, expect, it } from "vitest";

import { customIconUrl } from "./customIcon";

const hash = "a".repeat(64);

describe("customIconUrl", () => {
  it("serves a hashed image from the icon scheme", () => {
    expect(customIconUrl(`${hash}.png`)).toBe(`portolano-icon://localhost/${hash}.png`);
  });

  it("refuses a path that leaves the icon directory", () => {
    expect(customIconUrl(`../${hash}.png`)).toBeUndefined();
  });

  it("refuses an extension outside the accepted ones", () => {
    expect(customIconUrl(`${hash}.html`)).toBeUndefined();
  });

  it("refuses a name that is not a SHA-256", () => {
    expect(customIconUrl("logo.svg")).toBeUndefined();
  });
});
