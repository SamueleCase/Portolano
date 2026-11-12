import { describe, expect, it } from "vitest";

import { visibleText } from "./visibleText";

// The cases of `portolano-cli` `output::escape` (DEC-026.4): the same input
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

  it("keeps other text as it is, emoji and accents included", () => {
    expect(visibleText("📁 Clienti/ACME àè")).toBe("📁 Clienti/ACME àè");
  });
});
