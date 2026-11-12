/**
 * Strings of the document come from untrusted sources (DEC-024): a name, a
 * host or a note could carry control characters, or bidi controls that show
 * the text in another order than it has. Every component that shows them
 * passes them through here, so they read exactly as the CLI prints them
 * (`portolano-cli` `output::escape::visible`, DEC-026.4).
 */

/** C0 controls, DEL and C1 controls: Rust's `char::is_control` (category Cc). */
function isControl(codePoint: number): boolean {
  return codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f);
}

/** Embeddings and overrides, isolates, the marks and the Arabic letter mark. */
function isBidiControl(codePoint: number): boolean {
  return (
    (codePoint >= 0x202a && codePoint <= 0x202e) ||
    (codePoint >= 0x2066 && codePoint <= 0x2069) ||
    codePoint === 0x200e ||
    codePoint === 0x200f ||
    codePoint === 0x061c
  );
}

/** `text` with each control and bidi control written as `\u{1b}`. */
export function visibleText(text: string): string {
  let shown = "";
  for (const character of text) {
    const codePoint = character.codePointAt(0) ?? 0;
    shown +=
      isControl(codePoint) || isBidiControl(codePoint)
        ? `\\u{${codePoint.toString(16)}}`
        : character;
  }
  return shown;
}
