import { describe, expect, it } from "vitest";

import { applyAppearance } from "./appearance";
import { applyTokens, tokenStylesheet } from "./tokens";

describe("tokens", () => {
  it("define the panel radius of 14 px (ambiguity 2)", () => {
    expect(tokenStylesheet()).toContain("--radius-panel:14px;");
  });

  it("shrink rows and text in the compact density (ambiguity 8)", () => {
    expect(tokenStylesheet()).toContain(
      ':root[data-density="compact"]{--size-row:26px;--font-size-base:12.5px;}',
    );
  });

  it("turn movements into fades of 120 ms with the preference or the system setting", () => {
    const sheet = tokenStylesheet();

    expect(sheet).toMatch(/:root\[data-motion="reduced"\]\{--duration-fast:120ms;/);
    expect(sheet).toMatch(
      /@media \(prefers-reduced-motion: reduce\)\{:root\{--duration-fast:120ms;/,
    );
  });

  it("are installed once in the document", () => {
    applyTokens(document.documentElement);
    applyTokens(document.documentElement);

    expect(document.querySelectorAll("style#portolano-tokens")).toHaveLength(1);
  });
});

describe("applyAppearance", () => {
  it("marks the root with the density and reduced motion", () => {
    const root = document.createElement("div");

    applyAppearance(root, { density: "compact", reduceMotion: true });

    expect(root.dataset).toMatchObject({ density: "compact", motion: "reduced" });
  });

  it("removes the reduced motion mark when the preference is off", () => {
    const root = document.createElement("div");
    applyAppearance(root, { density: "normal", reduceMotion: true });

    applyAppearance(root, { density: "normal", reduceMotion: false });

    expect(root.dataset["motion"]).toBeUndefined();
  });
});
