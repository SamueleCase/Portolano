import { describe, expect, it } from "vitest";

import { isControl, JOINERS, visibleText } from "./visibleText";

// The cases of `portolano_core::text` (DEC-026.4, DEC-047.2): the same input
// must read the same in the app and in the terminal.
describe("visibleText", () => {
  it("escapes C0 controls, DEL and C1 controls like the CLI", () => {
    expect(visibleText("web\u001b[2J01\r\n\u0007\u007f\u009b")).toBe(
      "web\\u{1b}[2J01\\u{d}\\u{a}\\u{7}\\u{7f}\\u{9b}",
    );
  });

  it("escapes the bidi controls like the CLI", () => {
    // Built from code points: the characters themselves would hide in the source.
    const text = ["a", 0x202e, "b", 0x2066, "c", 0x200f, "d", 0x061c, "e"]
      .map((part) => (typeof part === "number" ? String.fromCodePoint(part) : part))
      .join("");

    expect(visibleText(text)).toBe("a\\u{202e}b\\u{2066}c\\u{200f}d\\u{61c}e");
  });

  it("escapes the window title sequence of a terminal", () => {
    expect(visibleText("Doc\u001b]0;x\u0007")).toBe("Doc\\u{1b}]0;x\\u{7}");
  });

  it("escapes NUL and the tab", () => {
    expect(visibleText("a\u0000b\tc")).toBe("a\\u{0}b\\u{9}c");
  });

  it("escapes the separators of lines and paragraphs like the CLI", () => {
    const text = String.fromCodePoint(0x61, 0x2028, 0x62, 0x2029, 0x63);

    expect(visibleText(text)).toBe("a\\u{2028}b\\u{2029}c");
  });

  it("escapes the zero-width and format characters like the CLI", () => {
    const text = String.fromCodePoint(0x77, 0x200b, 0x65, 0x62, 0x2060, 0x30, 0xfeff, 0x78);

    expect(visibleText(text)).toBe("w\\u{200b}eb\\u{2060}0\\u{feff}x");
  });

  it("keeps the joiners of a composed emoji, as the CLI", () => {
    // A family: man, ZWJ, woman, ZWJ, girl.
    const family = `Famiglia ${String.fromCodePoint(0x1f468, 0x200d, 0x1f469, 0x200d, 0x1f467)}`;

    expect(visibleText(family)).toBe(family);
  });

  it("escapes exactly the format characters and separators of Unicode, but the joiners", () => {
    const unicode = /[\p{Cf}\p{Zl}\p{Zp}]/u;
    const differ: string[] = [];
    for (let codePoint = 0; codePoint <= 0x10ffff; codePoint++) {
      if ((codePoint >= 0xd800 && codePoint <= 0xdfff) || isControl(codePoint)) continue;
      const character = String.fromCodePoint(codePoint);
      // The one exception of the rule: ZWJ and ZWNJ stay (DEC-047.5).
      const expected = unicode.test(character) && !JOINERS.has(codePoint);
      if ((visibleText(character) !== character) !== expected) {
        differ.push(codePoint.toString(16));
      }
    }

    expect(differ).toEqual([]);
  });

  it("keeps other text as it is, emoji and accents included", () => {
    expect(visibleText("📁 Clienti/ACME àè")).toBe("📁 Clienti/ACME àè");
  });
});
